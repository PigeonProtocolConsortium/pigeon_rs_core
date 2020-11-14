#![allow(unused)]
use regex;
use std::fs;
use std::io::{prelude::*, Error, ErrorKind};
use std::path;

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
            path: Store::normalize_path(path),
            buffer: vec![],
        }
    }

    pub fn open(&mut self) -> Result<(), Error> {
        if self.path.is_empty() {
            return Err(Error::new(ErrorKind::InvalidInput, "empty path for store"));
        }
        let p = self.path.clone();
        Store::ensure_dir(p.clone());
        let mut f = fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(p.clone())?;
        match fs::read(p) {
            Ok(buff) => {
                self.buffer = buff;
                Ok(())
            }
            Err(err) => Err(err),
        }
    }

    pub fn save(&mut self) -> Result<(), Error> {
        Store::ensure_dir(self.path.clone());
        fs::write(
            Store::normalize_path(self.path.clone()),
            self.buffer.clone(),
        )
    }

    fn normalize_path(p: String) -> String {
        let re = regex::Regex::new(r"[\\,/]+").unwrap();
        let sep = path::MAIN_SEPARATOR.to_string();
        re.replace_all(p.as_str(), sep.as_str()).to_string()
    }

    fn ensure_dir(p: String) {
        let np = Store::normalize_path(p);
        let pa = path::PathBuf::from(np);
        if !pa.exists() {
            fs::create_dir_all(pa.parent().unwrap());
        };
    }
}

#[cfg(test)]
mod tests {
    use super::Store;
    use regex;
    use std::{fs, io, iter::FromIterator, path, path::MAIN_SEPARATOR, time::SystemTime};

    const TMP_DIR: [&'static str; 5] = ["/tmp", "pigeon_core", "test_data", "storage", "store"];

    fn format_file_name(name: &str) -> String {
        let sep = String::from(MAIN_SEPARATOR.clone());
        let mut pc: Vec<&str> = vec![];
        for dir in TMP_DIR.to_vec() {
            pc.push(dir);
        }
        pc.push(name);
        pc.join(&sep)
    }

    fn setup_tmp_dir() -> Result<(), io::Error> {
        fs::create_dir_all(TMP_DIR.join(MAIN_SEPARATOR.to_string().as_str()))
    }

    fn setup_tmp_file(name: &str, buf: Option<Vec<u8>>) -> Result<String, io::Error> {
        setup_tmp_dir()?;
        let t = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        let now = format!("{}-{:?}", name, t);
        let path = format_file_name(&now);
        let result = match buf {
            Some(c) => fs::write(path.clone(), c),
            None => fs::write(path.clone(), ""),
        };
        match result {
            Err(err) => Err(err),
            _ => Ok(path),
        }
    }

    fn teardown_tmp_file(name: &str) {
        let _ = fs::remove_file(format_file_name(name));
    }

    #[test]
    fn returns_new_store() {
        let s = Store::new();
        let e = Store {
            path: String::default(),
            buffer: vec![],
        };
        assert_eq!(e, s);
    }

    #[test]
    fn returns_store_with_path() {
        let path = String::from("foo");
        let s = Store::from(path.clone());
        let e = Store {
            path,
            buffer: vec![],
        };
        assert_eq!(e, s);
    }

    #[test]
    fn store_open_errors_with_empty_path() {
        let mut s = Store::new();
        let resp = s.open();
        assert_eq!(true, resp.is_err());
        let e = resp.unwrap_err();
        assert_eq!(io::ErrorKind::InvalidInput, e.kind());
    }

    #[test]
    fn store_open_creates_new_file() {
        fn setup_path() -> String {
            let name = "store_open_creates_new_file";
            let t = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap();
            let now = format!("{}-{:?}", name, t);
            format_file_name(&now)
        }
        let path = setup_path();
        let mut s = Store::from(path.clone());
        let resp = s.open();
        assert_eq!(false, resp.is_err());
        let eb: Vec<u8> = vec![];
        assert_eq!(eb, s.buffer);
        teardown_tmp_file(&*path);
    }

    #[test]
    fn store_open_loads_existing_file() {
        let buf: Vec<u8> = "i exist".as_bytes().to_vec();
        let p = setup_tmp_file("store_load_existing", Some(buf.clone()));
        assert_eq!(true, p.is_ok());
        let path = p.unwrap();
        let mut s = Store::from(path.clone());
        let resp = s.open();
        assert_eq!(true, resp.is_ok());
        assert_eq!(buf, s.buffer);
        teardown_tmp_file(&*path);
    }

    #[test]
    fn store_save_create_new_file() {
        fn setup_path() -> String {
            let name = "store_save_create_new_file";
            let t = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap();
            let now = format!("{}-{:?}", name, t);
            format_file_name(&now)
        }
        let path = setup_path();
        let buf = "It's a strange time to be created".as_bytes().to_vec();
        let mut s = Store::from(path.clone());
        s.buffer = buf.clone();
        assert_eq!(buf.clone(), s.buffer);
        let mut resp = s.save();
        assert_eq!(true, resp.is_ok());

        let mut s = Store::from(path.clone());
        let mut resp = s.open();
        assert_eq!(true, resp.is_ok());
        assert_eq!(buf, s.buffer);
        teardown_tmp_file(&*path);
    }

    #[test]
    fn store_save_updates_existing_file() {
        let buf: Vec<u8> = "i exist".as_bytes().to_vec();
        fn setup_buffer(buf: Vec<u8>) -> String {
            let p = setup_tmp_file("store_save_updates_existing_file", Some(buf.clone()));
            assert_eq!(true, p.is_ok());
            p.unwrap()
        }
        fn setup_store(path: String) -> Store {
            let mut s = Store::from(path);
            let resp = s.open();
            assert_eq!(true, resp.is_ok());
            s
        }
        let path = setup_buffer(buf.clone());
        let mut s = setup_store(path.clone());
        assert_eq!(buf.clone(), s.buffer);

        let new_buf = "I exist!".as_bytes().to_vec();
        s.buffer = new_buf.clone();
        let resp = s.save();
        assert_eq!(true, resp.is_ok());

        let mut s = setup_store(path.clone());
        assert_ne!(buf, s.buffer);
        assert_eq!(new_buf, s.buffer);
        teardown_tmp_file(&*path);
    }

    #[test]
    fn store_normalize_path() {
        let re = regex::Regex::new(r"[\\,/]+").unwrap();
        let sep = "\\";
        let op = format_file_name("store_normalize_path");
        let e = op.clone();
        let mut a = op.clone().to_owned();
        println!("{}, {}, {}", op.clone(), e.clone(), a.clone());
        a = re.replace_all(a.as_str(), sep).to_string();
        assert_ne!(e, a);

        let r = Store::normalize_path(a.clone().to_owned());
        assert_ne!(a, r);
        assert_eq!(e, r);
    }
}
