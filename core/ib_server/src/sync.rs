use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::Path,
};

use rusqlite::Connection;

use crate::{
    db::{get_filename_uid, remove_file},
    IbFile,
};

// TODO: Delete file requests

pub fn create_file(uid: String, id: String, filename: String) -> bool {
    let conn = Connection::open("./data/files.db").unwrap();
    match conn.execute(
        "INSERT INTO files (id, uid, filename) VALUES (?1, ?2, ?3)",
        &[&id, &uid, &filename],
    ) {
        Ok(_) => true,
        Err(_) => false,
    }
}

pub fn delete_file(uid: String, id: String) -> bool {
    let db_file = match get_filename_uid(id.clone()) {
        Some(f) => f,
        None => return false,
    };

    if db_file.uid != uid {
        return false;
    }

    remove_file(id);

    let path = Path::new("data").join(uid.clone()).join(db_file.filename);
    let _ = fs::remove_file(path);

    true
}

/// Writes `code` as the contents of file `id`, if it belongs to `uid`. Returns
/// whether it was written.
pub fn sync_file(uid: String, id: String, code: String) -> bool {
    let path = Path::new("data").join(uid.clone());

    // create userdir
    let _ = fs::create_dir_all(path.clone());
    let db_file = match get_filename_uid(id) {
        Some(f) => f,
        None => return false,
    };

    if db_file.uid != uid {
        return false;
    }

    let path = path.join(db_file.filename);
    let file = if fs::metadata(path.clone()).is_ok() {
        OpenOptions::new().write(true).truncate(true).open(path)
    } else {
        File::create_new(path)
    };

    let mut file = match file {
        Ok(f) => f,
        Err(_) => return false,
    };

    // written as is: a trailing newline would come back on the next load and
    // grow the file by one line every time it is saved
    file.write_all(code.as_bytes()).is_ok()
}

pub fn get_files(uid: String) -> Vec<IbFile> {
    let conn = Connection::open("./data/files.db").unwrap();
    let mut query = conn.prepare("SELECT * FROM files WHERE uid = ?").unwrap();

    let rows = query
        .query_map([uid.clone()], |row| {
            let id = row.get::<_, String>(0)?;
            let filename = row.get::<_, String>(2)?;
            Ok((id, filename))
        })
        .unwrap();

    let mut files: Vec<IbFile> = Vec::new();
    let path = Path::new("data").join(uid);

    for row in rows {
        match row {
            Ok((id, filename)) => {
                let file_path = path.join(filename.clone());
                let contents = if !file_path.exists() {
                    "".to_string()
                } else {
                    fs::read_to_string(file_path).unwrap()
                };

                let ib_file = IbFile {
                    id: id,
                    filename: filename,
                    contents: contents,
                };

                files.push(ib_file)
            }
            Err(_) => {
                return vec![];
            }
        }
    }

    files
}
