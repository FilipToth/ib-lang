use std::{fs::{self, File, OpenOptions}, io::Write, path::Path};

use crate::IbFile;

pub fn sync_file(uid: String, file: String, code: String) {
    let path = Path::new("data")
        .join(uid);

    // create userdir
    let _ = fs::create_dir_all(path.clone());

    let path = path.join(file);

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
    let path = Path::new("data")
        .join(uid);

    let files_entries = match fs::read_dir(path.clone()) {
        Ok(f) => f,
        Err(_) => return Vec::new()
    };

    let mut files: Vec<IbFile> = Vec::new();
    for file in files_entries {
        let filename = file.unwrap().file_name().to_str().unwrap().to_string();

        let file_path = path.join(filename.clone());
        let contents = fs::read_to_string(file_path).unwrap();

        let ib_file = IbFile {
            filename: filename,
            contents: contents
        };

        files.push(ib_file)
    }

    files
}