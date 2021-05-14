use rand::random;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use crate::{Error, random_pw};

fn conf_path(cert: &PathBuf) -> PathBuf {
    cert.parent().unwrap_or(
        &PathBuf::from("/tmp")
    ).join(format!("ca-{}.conf", random::<u128>()))
}

pub fn generate_root(id: &str, key: &PathBuf, cert: &PathBuf) -> Result<String, Error> {
    let pw = random_pw();
    let status = Command::new("/usr/bin/certtool")
        .arg("--generate-privkey")
        .arg("--key-type").arg("ed25519")
        .arg("--pkcs-cipher").arg("aes-256")
        .arg("--password").arg(&pw)
        .arg("--outfile").arg(&key)
        .status()?;
    if !status.success() {
        return Err(Error::CertError());
    }
    let conf = conf_path(&cert);
    fs::write(&conf, format!(r"dn=cn={}
expiration_days=1825
ca
cert_signing_key
crl_signing_key", &id))?;
    let status = Command::new("/usr/bin/certtool")
        .env("GNUTLS_PIN", &pw)
        .arg("--generate-self-signed")
        .arg("--template").arg(&conf)
        .arg("--load-privkey").arg(&key)
        .arg("--outfile").arg(&cert)
        .status()?;
    fs::remove_file(conf)?;
    if !status.success() {
        return Err(Error::CertError());
    }
    Ok(pw)
}

pub fn generate(key: &PathBuf) -> Result<(), Error> {
    let status = Command::new("/usr/bin/certtool")
        .arg("--generate-privkey")
        .arg("--key-type").arg("ed25519")
        .arg("--no-text")
        .arg("--outfile").arg(&key)
        .status()?;
    if !status.success() {
        return Err(Error::CertError());
    }
    Ok(())
}

pub fn sign_ca(
    id: &str,
    cacert: &PathBuf,
    cakey: &PathBuf,
    key: &PathBuf,
    cert: &PathBuf,
) -> Result<(), Error> {
    let conf = conf_path(&cert);
    fs::write(&conf, format!("dn=cn={}
expiration_days=365
ca
cert_signing_key
crl_signing_key
signing_key
tls_www_client
tls_www_server", &id))?;
    let status = Command::new("/usr/bin/certtool")
        .arg("--generate-certificate")
        .arg("--template").arg(&conf)
        .arg("--ask-pass")
        .arg("--load-privkey").arg(&key)
        .arg("--load-ca-certificate").arg(&cacert)
        .arg("--load-ca-privkey").arg(&cakey)
        .arg("--outfile").arg(&cert)
        .status()?;
    fs::remove_file(conf)?;
    if !status.success() {
        return Err(Error::CertError());
    }
    Ok(())
}

pub fn sign(
    id: &str,
    cacert: &PathBuf,
    cakey: &PathBuf,
    csr: &PathBuf,
    cert: &PathBuf,
) -> Result<(), Error> {
    let conf = conf_path(&cert);
    fs::write(&conf, format!("dn=cn={}
expiration_days=365
signing_key
tls_www_client
tls_www_server
path_len=2", &id))?;
    let status = Command::new("/usr/bin/certtool")
        .arg("--generate-certificate")
        .arg("--template").arg(&conf)
        .arg("--load-request").arg(&csr)
        .arg("--load-ca-certificate").arg(&cacert)
        .arg("--load-ca-privkey").arg(&cakey)
        .arg("--outfile").arg(&cert)
        .status()?;
    fs::remove_file(conf)?;
    if !status.success() {
        return Err(Error::CertError());
    }
    Ok(())
}
