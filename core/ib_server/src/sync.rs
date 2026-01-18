use std::{
    collections::HashMap,
    fs::{self, File},
    io::Write,
    path::Path,
    sync::Mutex,
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

/// The sequence number of the newest save written to each file, by file id.
///
/// A client that gives up waiting on a save sends the next one regardless, and
/// the one it gave up on can still arrive after it. Its number is lower, so it
/// is turned away rather than writing older contents over newer ones. Holding
/// the lock through the write also keeps two saves of a file from interleaving.
static LATEST_SAVES: Mutex<Option<HashMap<String, u64>>> = Mutex::new(None);

/// Writes `code` as the contents of file `id`, if it belongs to `uid`. Returns
/// whether it was written.
///
/// With a `seq`, a save numbered no higher than one already written is refused.
pub fn sync_file(uid: String, id: String, code: String, seq: Option<u64>) -> bool {
    let dir = Path::new("data").join(uid.clone());

    // create userdir
    let _ = fs::create_dir_all(dir.clone());
    let db_file = match get_filename_uid(id.clone()) {
        Some(f) => f,
        None => return false,
    };

    if db_file.uid != uid {
        return false;
    }

    // a panic while holding the lock leaves the map as it was, which is fine
    let mut latest = LATEST_SAVES.lock().unwrap_or_else(|e| e.into_inner());
    let latest = latest.get_or_insert_with(HashMap::new);

    if let Some(seq) = seq {
        if latest.get(&id).is_some_and(|&newest| seq <= newest) {
            return false;
        }
    }

    // written beside the file and renamed over it, so a failed write leaves
    // the previous contents rather than a truncated file
    let path = dir.join(&db_file.filename);
    let temp = dir.join(format!(".{}.saving", db_file.filename));

    // written as is: a trailing newline would come back on the next load and
    // grow the file by one line every time it is saved
    let written = File::create(&temp)
        .and_then(|mut file| {
            file.write_all(code.as_bytes())?;
            file.sync_all()
        })
        .and_then(|_| fs::rename(&temp, &path));

    if written.is_err() {
        let _ = fs::remove_file(&temp);
        return false;
    }

    if let Some(seq) = seq {
        latest.insert(id, seq);
    }

    true
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
