use std::{
    collections::HashMap,
    fs::{self, File},
    io::Write,
    path::Path,
    sync::{Mutex, MutexGuard},
};

use rusqlite::Connection;

use crate::{
    db::{filename_taken, get_filename_uid, remove_file, set_filename},
    IbFile,
};

// TODO: Delete file requests

/// Why a file name is not allowed, if it is not. Names become paths on disk,
/// so anything that could step out of the user's folder is refused, as is
/// anything the editor itself would not produce.
fn invalid_filename(filename: &str) -> Option<String> {
    let Some(stem) = filename.strip_suffix(".ib") else {
        return Some("File names must end in .ib.".to_string());
    };

    if stem.trim().is_empty() {
        return Some("File names cannot be empty.".to_string());
    }

    if filename.chars().count() > 100 {
        return Some("File names can be at most 100 characters long.".to_string());
    }

    if stem
        .chars()
        .any(|c| c == '.' || c == '/' || c == '\\' || c.is_control())
    {
        return Some("File names cannot contain periods or slashes.".to_string());
    }

    None
}

fn taken_message(filename: &str) -> String {
    format!("A file named {filename} already exists.")
}

pub fn create_file(uid: String, id: String, filename: String) -> Result<(), String> {
    if let Some(reason) = invalid_filename(&filename) {
        return Err(reason);
    }

    let _files = lock_files();

    // two files of one name would be stored at one path, each overwriting
    // the other
    if filename_taken(&uid, &filename, &id) {
        return Err(taken_message(&filename));
    }

    let conn = Connection::open("./data/files.db").unwrap();
    conn.execute(
        "INSERT INTO files (id, uid, filename) VALUES (?1, ?2, ?3)",
        [&id, &uid, &filename],
    )
    .map(|_| ())
    .map_err(|_| "The file could not be created.".to_string())
}

/// Renames file `id` to `filename`, on disk and in the database.
pub fn rename_file(uid: String, id: String, filename: String) -> Result<(), String> {
    if let Some(reason) = invalid_filename(&filename) {
        return Err(reason);
    }

    // held so a save cannot write to the old name after it has moved
    let _files = lock_files();

    let db_file = match get_filename_uid(id.clone()) {
        Some(f) if f.uid == uid => f,
        _ => return Err("The file no longer exists.".to_string()),
    };

    if db_file.filename == filename {
        return Ok(());
    }

    if filename_taken(&uid, &filename, &id) {
        return Err(taken_message(&filename));
    }

    let dir = Path::new("data").join(&uid);
    let old_path = dir.join(&db_file.filename);
    let new_path = dir.join(&filename);

    // a file never saved has nothing on disk to move
    let moved = old_path.exists();
    if moved && fs::rename(&old_path, &new_path).is_err() {
        return Err("The file could not be renamed.".to_string());
    }

    if !set_filename(&id, &filename) {
        // put it back, so the database and the disk agree
        if moved {
            let _ = fs::rename(&new_path, &old_path);
        }
        return Err("The file could not be renamed.".to_string());
    }

    Ok(())
}

pub fn delete_file(uid: String, id: String) -> bool {
    let _files = lock_files();

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

/// Serialises every change to the files: saves, renames, creates and deletes.
///
/// It also holds the sequence number of the newest save written to each file,
/// by file id. A client that gives up waiting on a save sends the next one
/// regardless, and the one it gave up on can still arrive after it. Its number
/// is lower, so it is turned away rather than writing older contents over
/// newer ones.
static FILES: Mutex<Option<HashMap<String, u64>>> = Mutex::new(None);

fn lock_files() -> MutexGuard<'static, Option<HashMap<String, u64>>> {
    // a panic while holding the lock leaves the map as it was, which is fine
    FILES.lock().unwrap_or_else(|e| e.into_inner())
}

/// Writes `code` as the contents of file `id`, if it belongs to `uid`. Returns
/// whether it was written.
///
/// With a `seq`, a save numbered no higher than one already written is refused.
pub fn sync_file(uid: String, id: String, code: String, seq: Option<u64>) -> bool {
    let dir = Path::new("data").join(uid.clone());

    // create userdir
    let _ = fs::create_dir_all(dir.clone());

    // taken before the file is looked up, so a rename cannot move it between
    // the lookup and the write
    let mut files = lock_files();
    let latest = files.get_or_insert_with(HashMap::new);

    let db_file = match get_filename_uid(id.clone()) {
        Some(f) => f,
        None => return false,
    };

    if db_file.uid != uid {
        return false;
    }

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

#[cfg(test)]
mod tests {
    use super::invalid_filename;

    #[test]
    fn accepts_names_the_editor_makes() {
        assert_eq!(invalid_filename("main.ib"), None);
        assert_eq!(invalid_filename("my program 2.ib"), None);
        assert_eq!(invalid_filename("úloha_1-a.ib"), None);
    }

    /// Names become paths, so nothing may lead out of the user's folder.
    #[test]
    fn refuses_names_that_could_escape_the_folder() {
        assert!(invalid_filename("../other/main.ib").is_some());
        assert!(invalid_filename("..ib").is_some());
        assert!(invalid_filename("a/b.ib").is_some());
        assert!(invalid_filename("a\\b.ib").is_some());
        assert!(invalid_filename("a\nb.ib").is_some());
    }

    #[test]
    fn refuses_names_the_editor_would_not_make() {
        assert!(invalid_filename("main").is_some());
        assert!(invalid_filename("main.txt").is_some());
        assert!(invalid_filename(".ib").is_some());
        assert!(invalid_filename("  .ib").is_some());
        assert!(invalid_filename("a.b.ib").is_some());
        assert!(invalid_filename(&format!("{}.ib", "a".repeat(100))).is_some());
    }
}
