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

/// How many files `uid` has. `None` when the count could not be read, which
/// callers treat as being at the cap rather than under it.
pub fn count_files(uid: &str) -> Option<usize> {
    let conn = Connection::open(DB_PATH).ok()?;

    conn.query_row("SELECT COUNT(*) FROM files WHERE uid = ?1", [uid], |row| {
        row.get::<_, i64>(0)
    })
    .ok()
    .map(|count| count as usize)
}

/// Whether `uid` has a file named `filename` other than file `except`.
pub fn filename_taken(uid: &str, filename: &str, except: &str) -> bool {
    let conn = Connection::open(DB_PATH).unwrap();

    // a failed query answers "taken", so a name is never reused on a guess
    conn.query_row(
        "SELECT COUNT(*) FROM files WHERE uid = ?1 AND filename = ?2 AND id != ?3",
        [uid, filename, except],
        |row| row.get::<_, i64>(0),
    )
    .map_or(true, |count| count > 0)
}

pub fn set_filename(id: &str, filename: &str) -> bool {
    let conn = Connection::open(DB_PATH).unwrap();
    conn.execute(
        "UPDATE files SET filename = ?1 WHERE id = ?2",
        [filename, id],
    )
    .is_ok_and(|changed| changed == 1)
}

pub fn remove_file(id: String) {
    let conn = Connection::open(DB_PATH).unwrap();
    let _ = conn.execute("DELETE FROM files WHERE id = ?1", [id]);
}
