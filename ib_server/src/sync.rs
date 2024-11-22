use std::{fs::{self, File, OpenOptions}, io::Write, path::Path};

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