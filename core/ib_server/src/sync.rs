use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::Path,
};

use rusqlite::Connection;

use crate::IbFile;

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

pub fn sync_file(uid: String, id: String, code: String) {
    let path = Path::new("data").join(uid.clone());

    // create userdir
    let _ = fs::create_dir_all(path.clone());

    let conn = Connection::open("./data/files.db").unwrap();

    let mut query = conn
        .prepare("SELECT uid, filename FROM files WHERE id = ?")
        .unwrap();

    let (file_uid, filename): (String, String) = match query.query_row([id], |row| {
        Ok((
            row.get::<_, Option<String>>(0)?,
            row.get::<_, Option<String>>(1)?,
        ))
    }) {
        Ok((uid, filename)) => {
            let uid = match uid {
                Some(value) => value,
                None => {
                    eprintln!("UID is None");
                    return;
                }
            };

            let filename = match filename {
                Some(value) => value,
                None => {
                    eprintln!("Filename is None");
                    return;
                }
            };

            (uid, filename)
        }
        Err(e) => {
            eprintln!("Query failed: {}", e);
            return;
        }
    };

    if file_uid != uid {
        return;
    }

    let path = path.join(filename);

    let mut file = if fs::metadata(path.clone()).is_ok() {
        let file = OpenOptions::new().write(true).truncate(true).open(path);

        let file = match file {
            Ok(f) => f,
            Err(_) => return,
        };

        file
    } else {
        match File::create_new(path) {
            Ok(f) => f,
            Err(_) => return,
        }
    };

    let _ = writeln!(file, "{}", code);
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
