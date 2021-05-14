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
