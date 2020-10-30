use std::fs;
use std::io::{prelude::*, Error, ErrorKind};

#[derive(Debug, PartialEq)]
pub struct Store {
    path: String,
    buffer: Vec<u8>,
}

impl Store {
    pub fn new() -> Self {
        Store {
            path: String::default(),
            buffer: vec![],
        }
    }

    pub fn from(path: String) -> Store {
        Store {
            path,
            buffer: vec![],
        }
    }

    pub fn open(&mut self) -> Result<(), Error> {
        if self.path.is_empty() {
            return Err(Error::new(ErrorKind::InvalidInput, "empty path for store"));
        }
        let mut f = fs::OpenOptions::new()
            .append(true)
            .read(true)
            .create(true)
            .open(&self.path)?;
        match f.read(&mut self.buffer) {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Store;
    use std::{fs, io, time::SystemTime};

    const TMP_DIR: &str = "/tmp/pigeon_core/test_data/storage/store";

    fn format_file_name(name: &str) -> String {
        format!("{}/{}", TMP_DIR, name)
    }

    fn setup_tmp_dir() -> Result<(), io::Error> {
        fs::create_dir_all(TMP_DIR)
    }

    fn teardown_tmp_dir() -> Result<(), io::Error> {
        fs::remove_dir(TMP_DIR)
    }

    fn setup_tmp_file(name: &str, buf: Option<Vec<u8>>) -> Result<(), io::Error> {
        setup_tmp_dir()?;
        let t = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        let now = format!("{}-{:?}", name, t);
        let path = format_file_name(&now);
        match buf {
            Some(c) => fs::write(path, c),
            None => fs::write(path, ""),
        }
    }

    fn teardown_tmp_file(name: &str) {
        let _ = fs::remove_file(format_file_name(name));
    }

    #[test]
    fn test_store_new() {
        let s = Store::new();
        let e = Store {
            path: String::default(),
            buffer: vec![],
        };
        assert_eq!(e, s);
    }

    #[test]
    fn test_store_from() {
        let path = String::from("foo");
        let s = Store::from(path.to_owned());
        let e = Store {
            path: path,
            buffer: vec![],
        };
        assert_eq!(e, s);
    }

    #[test]
    fn test_store_open_no_path() {
        let mut s = Store::new();
        let resp = s.open();
        assert_eq!(true, resp.is_err());
        let e = resp.unwrap_err();
        assert_eq!(io::ErrorKind::InvalidInput, e.kind());
    }

    #[test]
    fn test_store_open_create() {
        setup_tmp_dir().unwrap();
        let t = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        let now = format!("{:?}", t);
        let path = format_file_name(&now);
        let mut s = Store::from(path);
        let resp = s.open();
        assert_eq!(false, resp.is_err());
        let eb: Vec<u8> = vec![];
        assert_eq!(eb, s.buffer);
        teardown_tmp_file(&now);
    }
}
