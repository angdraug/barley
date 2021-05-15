use std::fs;
use std::path::PathBuf;
use std::process::Command;

use crate::Error;

pub fn sign(id: &str, ca: &PathBuf, key: &PathBuf) -> Result<(), Error> {
    let status = Command::new("/usr/bin/ssh-keygen")
        .arg("-I").arg(&id)
        .arg("-s").arg(&ca)
        .arg("-h")
        .arg(&key)
        .status()?;
    match status.success() {
        true  => Ok(()),
        false => Err(Error::CertError()),
    }
}

pub fn authorized_keys(path: &PathBuf) -> Result<String, Error> {
    match fs::read(&path) {
        Ok(bytes)  => {
            let key = String::from_utf8_lossy(&bytes);
            Ok(format!("{}\ncert-authority {}", key, key))
        },
        Err(err) => Err(Error::IoError(format!("Failed to read {:?}: {}", path, err))),
    }
}
