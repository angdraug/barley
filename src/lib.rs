use actix_web::ResponseError;
use rand::random;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{error, fs, io, net, str, string};
use std::{convert::From, fmt::{Debug, Display, Formatter}, path::PathBuf};

pub mod ssh;
pub mod tls;

#[derive(Serialize)]
pub struct Certs {
    admin: String,
    host:  String,
    ca:    String,
    cert:  String,
}

#[derive(Deserialize)]
pub struct Registration {
    otp: String,
    ip:  net::IpAddr,
    ssh: String,
    csr: String,
}

#[derive(Clone)]
pub struct Data {
    home: PathBuf,
}

impl Data {
    pub fn new(home: PathBuf) -> Result <Self, Error> {
        if let Ok(m) = fs::metadata(&home) {
            if !m.is_dir() {
                return Err(Error::DataError(format!("Data directory {:?} is not a directory", home)));
            }
            if m.permissions().readonly() {
                return Err(Error::DataError(format!("Data directory {:?} is not writeable", home)));
            }
        } else {
            if let Err(err) = fs::create_dir(&home) {
                return Err(Error::DataError(format!("Failed to create data directory {:?}: {}", home, err)));
            }
        };
        Ok(Data { home })
    }

    pub fn reserve(&self, prefix: &str) -> Result<String, Error> {
        for i in NameCounter::new() {
            let name = format!("{}-{}", prefix, i);
            let path = self.home.join(&name);
            match fs::create_dir(&path) {
                Ok(_)  => return Ok(name),
                Err(err) => {
                    if let Err(_) = fs::metadata(&path) {
                        return Err(Error::DataError(format!("Failed to create {:?}: {}", path, err)));
                    }
                    // if path already exists, keep iterating
                }
            }
        }
        Err(Error::DataError(format!("Ran out of names for {} under {:?}", prefix, self.home)))
    }

    pub fn file(&self, name: &str) -> PathBuf {
        self.home.join(name)
    }

    pub fn read(&self, name: &str) -> Result<String, Error> {
        let path = self.file(&name);
        fs::read_to_string(&path)
            .or_else(|err| Err(Error::DataError(format!("Failed to read {:?}: {}", path, err))))
    }

    pub fn write(&self, name: &str, data: &str) -> Result<(), Error> {
        let path = self.file(&name);
        fs::write(&path, &data)
            .or_else(|err| Err(Error::DataError(format!("Failed to write {:?}: {}", path, err))))
    }
}

#[derive(Clone)]
pub struct Sower {
    pub ip: net::IpAddr,
    pub images: Data,
    data: Data,
}

impl Sower {
    pub fn new(dnsmasq: &str, image_dir: &str, data_dir: &str) -> Self {
        Self {
            ip: Self::detect_bind_ip(dnsmasq),
            images: Data::new(PathBuf::from(image_dir)).unwrap(),
            data: Data::new(PathBuf::from(data_dir)).unwrap(),
        }
    }

    pub fn binding(&self) -> String {
        format!("{}:8000", self.ip)
    }

    pub fn ipxe(&self) -> Result<String, Error> {
        let name = self.data.reserve("seed")?;
        Ok(Seed::new(&self.data, &name)?.ipxe())
    }

    pub fn init(&self, name: &str) -> Result<String, Error> {
        Seed::new(&self.data, &name)?.otp().map(|otp| {
            format!("SOWER={}\nOTP={}\n", &self.ip, otp)
        })
    }

    pub fn register(&self, name: &str, reg: &Registration) -> Result<Certs, Error> {
        let seed = Seed::new(&self.data, &name)?;
        seed.check_otp(&reg.otp)?;
        seed.write_ip(&reg.ip)?;
        let admin = ssh::authorized_keys(&self.data.file("admin.pub"))?;
        let host = seed.sign_ssh(&reg.ssh, &self.data.file("ca"))?;
        let ca = self.data.read("root.crt")?;
        let mut cert = seed.sign_tls(
            &reg.csr,
            &self.data.file("machine.crt"),
            &self.data.file("machine.key"),
        )?;
        cert.push_str(&self.data.read("machine.crt")?);
        Ok(Certs { admin, host, ca, cert })
    }

    fn parse_dnsmasq(conf: &str) -> net::IpAddr {
        match Regex::new(r"http://(?P<ip>.*?):\d+/").unwrap().captures_iter(conf).next() {
            Some(url) => {
                match url["ip"].parse() {
                    Ok(ip)   => ip,
                    Err(err) => panic!("Failed to parse '{}': {}", &url["ip"], err),
                }
            },
            None => panic!("Did not find an IP address in dnsmasq.conf"),
        }
    }

    fn detect_bind_ip(dnsmasq: &str) -> net::IpAddr {
        match fs::read_to_string(&dnsmasq) {
            Ok(conf) => Self::parse_dnsmasq(&conf),
            Err(err) => panic!("Failed to read {}: {}", dnsmasq, err),
        }
    }
}

pub struct Seed {
    name: String,
    data: Data,
}

impl Seed {
    pub fn new(home: &Data, name: &str) -> Result<Self, Error> {
        Ok(Seed {
            name: name.to_string(),
            data: Data::new(home.file(&name))?
        })
    }

    pub fn ipxe(&self) -> String {
        if let Err(err) = self.data.write("otp", &random_pw()) {
            eprintln!("Failed to write to {:?}: {}", self.data.file("otp"), err);
            // complain but let it boot anyway
        }
        format!(r"#!ipxe
kernel seed.vmlinuz rdinit=/lib/systemd/systemd systemd.hostname={} console=ttyS0
initrd seed.cpio.zst
initrd init/{} /etc/default/barley-seed
boot
", self.name, self.name)
    }

    pub fn otp(&self) -> Result<String, Error> {
        self.data.read("otp")
    }

    pub fn check_otp(&self, otp: &str) -> Result<(), Error> {
        if otp != self.otp()? {
            eprintln!("OTP mismatch for {}", &self.name);
            return Err(Error::OtpError());
        }
        fs::remove_file(self.data.file("otp"))?;
        Ok(())
    }

    pub fn write_ip(&self, ip: &net::IpAddr) -> Result<(), Error> {
        self.data.write("ip", &ip.to_string())
    }

    fn sign_ssh(&self, key: &str, ca: &PathBuf) -> Result<String, Error> {
        self.data.write("ssh.pub", key)?;
        ssh::sign(&self.name, &ca, &self.data.file("ssh.pub"))?;
        self.data.read("ssh-cert.pub")
    }

    fn sign_tls(&self, csr: &str, cacert: &PathBuf, cakey: &PathBuf) -> Result<String, Error> {
        fs::write(self.data.file("csr"), csr)?;
        tls::sign(
            &self.name,
            &cacert,
            &cakey,
            &self.data.file("csr"),
            &self.data.file("crt"),
        )?;
        self.data.read("crt")
    }
}

pub struct NameCounter {
    count: [u8; 8],
}

impl NameCounter {
    pub fn new() -> Self {
        NameCounter { count: [b'0'; 8] }
    }
}

impl Iterator for NameCounter {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        for digit in (0..8).rev() {
            match self.count[digit] {
                b'z' => { self.count[digit] = b'0'; continue; },
                b'9' => { self.count[digit] = b'a'; },
                _    => { self.count[digit] += 1; },
            };
            return Some(
                str::from_utf8(&self.count).unwrap()
                .trim_start_matches('0').to_owned()
            );
        }
        None
    }
}

pub fn random_pw() -> String {
    let pw: u128 = random();
    format!("{:0x}", pw)
}

fn table_line(fields: &[&str], widths: &[usize]) -> String {
    fields.iter().zip(widths.iter())
        .map(|(f, w)| format!("{:1$} ", f, w))
        .collect::<String>().trim_end().to_string()
}

pub fn print_table<'a, T, F>(list: &'a[T], plural: &str, headers: &[&str], fields: F)
where F: Fn(&'a T) -> Vec<&'a str> {
    if list.is_empty() {
        println!("No {}.", &plural);
        return;
    }
    let mut widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();
    for record in list {
        let f = &fields(&record);
        for i in 0..widths.len()-1 {
            if i > f.len()-1 {
                break;
            }
            if f[i].len() > widths[i] {
                widths[i] = f[i].len()
            }
        }
    }
    println!("{}", table_line(&headers, &widths));
    for record in list {
        println!("{}", table_line(&fields(&record), &widths));
    }
}

#[derive(Debug)]
pub enum Error {
    IoError(String),
    StrError(String),
    DataError(String),
    OtpError(),
    CertError(),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error: {:?}", self)
    }
}

impl error::Error for Error {}

impl ResponseError for Error {}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::IoError(err.to_string())
    }
}

impl From<str::Utf8Error> for Error {
    fn from(err: std::str::Utf8Error) -> Self {
        Error::StrError(err.to_string())
    }
}

impl From<string::FromUtf8Error> for Error {
    fn from(err: string::FromUtf8Error) -> Self {
        Error::StrError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counter() {
        let mut c = NameCounter::new();
        assert_eq!(c.next(), Some(String::from("1")));
        for _ in 0..3000 {
            c.next();
        }
        assert_eq!(c.next(), Some(String::from("2be")));
    }

    #[test]
    fn test_data() {
        let data = Data::new(PathBuf::from("/tmp")).unwrap();
        assert_eq!(data.file("foo"), PathBuf::from("/tmp/foo"));
    }

    #[test]
    fn test_parse_dnsmasq() {
        let ip = Sower::parse_dnsmasq("pxe-service=net:ipxe, X86PC,, http://127.0.0.1:8000/seed.ipxe");
        assert!(ip.is_ipv4());
    }

    #[test]
    #[should_panic]
    fn test_parse_dnsmasq_fail() {
        Sower::parse_dnsmasq("dhcp-range=127.0.0.1,proxy");
    }
}
