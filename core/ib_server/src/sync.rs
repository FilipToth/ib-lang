use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::Path,
};

use rusqlite::{Connection, OptionalExtension};

use crate::IbFile;

// TODO: Create file requests
// TODO: Delete file requests

pub fn sync_file(uid: String, id: String, code: String) {
    let path = Path::new("data").join(uid);

    // create userdir
    let _ = fs::create_dir_all(path.clone());

    let conn = Connection::open("./data/files.db").unwrap();

    let mut query = conn.prepare("SELECT filename FROM files WHERE id = ?").unwrap();
    let filename: String = match query.query_row([id], |row| row.get(0).optional()).unwrap() {
        Some(f) => f,
        None => return
    };

    let path = path.join(filename);

    let mut file = if fs::metadata(path.clone()).is_ok() {
        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(path);

        let file = match file {
            Ok(f) => f,
            Err(_) => return
        };

        file
    } else {
        match File::create_new(path) {
            Ok(f) => f,
            Err(_) => return
        }
    };

    let _ = writeln!(file, "{}", code);
}

pub fn get_files(uid: String) -> Vec<IbFile> {
    let path = Path::new("data").join(uid);

    let files_entries = match fs::read_dir(path.clone()) {
        Ok(f) => f,
        Err(_) => return Vec::new(),
    };

    let mut files: Vec<IbFile> = Vec::new();
    for file in files_entries {
        let filename = file.unwrap().file_name().to_str().unwrap().to_string();

        let file_path = path.join(filename.clone());
        let contents = fs::read_to_string(file_path).unwrap();

        let ib_file = IbFile {
            filename: filename,
            contents: contents,
        };

        files.push(ib_file)
    }

    files
}
