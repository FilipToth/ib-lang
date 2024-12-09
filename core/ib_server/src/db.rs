use rusqlite::Connection;

const DB_PATH: &str = "./data/files.db";

pub struct DBFilenameUid {
    pub filename: String,
    pub uid: String,
}

pub fn get_filename_uid(id: String) -> Option<DBFilenameUid> {
    let conn = Connection::open(DB_PATH).unwrap();
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
                    return None;
                }
            };

            let filename = match filename {
                Some(value) => value,
                None => {
                    eprintln!("Filename is None");
                    return None;
                }
            };

            (uid, filename)
        }
        Err(e) => {
            eprintln!("Query failed: {}", e);
            return None;
        }
    };

    let res = DBFilenameUid {
        filename: filename,
        uid: file_uid,
    };

    Some(res)
}

pub fn remove_file(id: String) {
    let conn = Connection::open(DB_PATH).unwrap();
    let _ = conn.execute("DELETE FROM files WHERE id = ?1", [id]);
}
