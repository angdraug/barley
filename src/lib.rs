use rand::random;
use regex::Regex;
use std::{fs, io, net, str};
use std::{convert::From, fmt::Debug, path::{Path, PathBuf}, process::Command};

const IMAGE_DIR: &str = "/srv/barley";
const DATA_DIR: &str = "/var/lib/barley";
const DNSMASQ: &str = "/etc/dnsmasq.d/barley.conf";

#[derive(Clone)]
pub struct Sower<'a> {
    pub ip: net::IpAddr,
    images: &'a Path,
}

impl<'a> Sower<'a> {
    pub fn new() -> Sower<'a> {
        check_data_dir();
        Sower {
            ip: detect_bind_ip(),
            images: &Path::new(IMAGE_DIR),
        }
    }

    pub fn binding(&self) -> String {
        format!("{}:8000", self.ip)
    }

    pub fn image(&self, name: &str) -> PathBuf {
        self.images.join(name)
    }
}

fn check_data_dir() {
    let m = match fs::metadata(DATA_DIR) {
        Ok(m) => m,
        Err(_) => panic!("Data directory {} not found", DATA_DIR),
    };
    if !m.is_dir() {
        panic!("Data directory {} is not a directory", DATA_DIR);
    }
    if m.permissions().readonly() {
        panic!("Data directory {} is not writeable", DATA_DIR);
    }
}

fn detect_bind_ip() -> net::IpAddr {
    match fs::read(DNSMASQ) {
        Ok(conf) => parse_dnsmasq(&str::from_utf8(&conf).unwrap()),
        Err(err) => panic!("Failed to read {}: {}", DNSMASQ, err),
    }
}

fn parse_dnsmasq(conf: &str) -> net::IpAddr {
    match Regex::new(r"http://(?P<ip>.*?):\d+/").unwrap().captures_iter(conf).next() {
        Some(url) => {
            match url["ip"].parse() {
                Ok(ip)   => ip,
                Err(err) => panic!("Failed to parse '{}': {}", &url["ip"], err),
            }
        },
        None => panic!("Failed to find an IPv6 address in {}", DNSMASQ),
    }
}

pub struct Seed {
    name: String,
}

impl Seed {
    pub fn new(name: String) -> Seed {
        Seed { name }
    }

    pub fn reserve() -> Seed {
        for i in NameCounter::new() {
            let name = format!("seed-{}", i);
            if fs::create_dir(Path::new(DATA_DIR).join(&name)).is_ok() {
                return Seed { name };
            }
        }
        panic!("Ran out of names")
    }

    pub fn ipxe(&self) -> String {
        let otp: u128 = random();
        if let Err(err) = fs::write(self.path("otp"), format!("{:0x}", otp)) {
            eprintln!("Failed to write to {:?}: {}", self.path("otp"), err);
            // complain but let it boot anyway
        }
        format!(r"#!ipxe
kernel seed.vmlinuz rdinit=/lib/systemd/systemd systemd.hostname={} console=ttyS0
initrd seed.cpio.gz
initrd init/{} /etc/default/barley-seed
boot
", self.name, self.name)
    }

    pub fn init(&self, sower_ip: net::IpAddr) -> String {
        match self.otp() {
            Ok(otp)  => {
                format!("SOWER={}\nOTP={}\n", sower_ip, otp)
            },
            Err(err) => {
                eprintln!("Failed to load OTP: {:?}", err);
                "".to_owned()
            },
        }
    }

    pub fn register(&self, otp: &str, ip: &net::IpAddr, ssh: &str) -> Result<String, Error> {
        if otp != self.otp()? {
            return Err(Error::OtpError());
        }
        fs::remove_file(self.path("otp"))?;
        fs::write(self.path("ip"), format!("{}", ip))?;
        fs::write(self.path("ssh.pub"), ssh)?;
        let status = Command::new("/usr/bin/ssh-keygen")
            .arg("-I").arg(&self.name)
            .arg("-s").arg("/var/lib/barley/ca")
            .arg("-h")
            .arg(self.path("ssh.pub"))
            .status()?;
        if !status.success() {
            return Err(Error::CertError());
        }
        let cert = fs::read(self.path("ssh-cert.pub"))?;
        let cert = str::from_utf8(&cert)?;
        Ok(cert.to_owned())
    }

    fn path(&self, file: &str) -> PathBuf {
        Path::new(DATA_DIR).join(&self.name).join(file)
    }

    fn otp(&self) -> Result<String, Error> {
        let otp = fs::read(self.path("otp"))?;
        let otp = str::from_utf8(&otp)?;
        Ok(otp.to_owned())
    }
}

struct NameCounter {
    count: [u8; 8],
}

impl NameCounter {
    fn new() -> NameCounter {
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

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    StrError(str::Utf8Error),
    OtpError(),
    CertError(),
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::IoError(error)
    }
}

impl From<str::Utf8Error> for Error {
    fn from(error: str::Utf8Error) -> Self {
        Error::StrError(error)
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
    fn test_seed() {
        let seed = Seed::new(String::from("foo"));
        assert_eq!(seed.name, "foo");
        assert_eq!(seed.path("bar"), Path::new("/var/lib/barley/foo/bar"));
    }

    #[test]
    fn test_parse_dnsmasq() {
        let ip = parse_dnsmasq("pxe-service=net:ipxe, X86PC,, http://127.0.0.1:8000/seed.ipxe");
        assert!(ip.is_ipv4());
    }

    #[test]
    #[should_panic]
    fn test_parse_dnsmasq_fail() {
        parse_dnsmasq("dhcp-range=127.0.0.1,proxy");
    }
}
