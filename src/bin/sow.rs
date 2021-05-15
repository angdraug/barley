use chrono::Local;
use std::{env, ffi, fs};
use std::cmp::Ordering;
use std::io::{Error, Write};
use std::path::PathBuf;
use std::process::{ChildStdout, Command, Stdio};
use std::time::SystemTime;
use structopt::StructOpt;
use version_compare::{CompOp, VersionCompare};

use barley::{Data, print_table, tls};

fn home() -> PathBuf {
    match env::var("HOME") {
        Ok(home) => PathBuf::from(&home),
        Err(_) => panic!("HOME environment variable is not set"),
    }
}

fn home_ssh() -> PathBuf {
    home().join(".ssh")
}

fn home_barley() -> PathBuf {
    home().join(".barley")
}

fn fields_home() -> PathBuf {
    home_barley().join("fields")
}

fn images_home() -> PathBuf {
    home_barley().join("images")
}

fn setup() {
    Data::new(home_barley()).unwrap();
    Data::new(fields_home()).unwrap();
    Data::new(images_home()).unwrap();
}

struct Field {
    name: String,
    modified: SystemTime,
}

impl Field {
    fn new(name: &str) -> Self {
        Field { name: name.to_string(), modified: SystemTime::now() }
    }

    fn from_dir(entry: &fs::DirEntry) -> Option<Self> {
        let metadata = entry.metadata().unwrap();
        if metadata.is_dir() {
            let name = entry.file_name().into_string().unwrap();
            let modified = metadata.modified().unwrap();
            Some(Field { name, modified })
        } else {
            None
        }
    }

    fn latest() -> Option<Self> {
        Self::all().max_by_key(|f| f.modified)
    }

    fn all() -> impl Iterator<Item=Field> {
        fs::read_dir(&fields_home()).unwrap()
            .filter_map(|entry| Self::from_dir(&entry.unwrap()))
    }

    fn create(&self, key: &PathBuf) -> String {
        let path = self.path();
        if let Ok(_) = fs::metadata(&path) {
            panic!("Field '{}' already exists", &self.name);
        }
        Data::new(path).unwrap();
        if let Err(err) = fs::copy(key, self.admin()) {
            panic!("Failed to copy admin public key {:?}: {}", key, err);
        }
        tls::generate_root(&self.name, &self.cakey(), &self.cacert()).unwrap()
    }

    fn path(&self) -> PathBuf {
        fields_home().join(&self.name)
    }

    fn file(&self, name: &str) -> PathBuf {
        self.path().join(&name)
    }

    fn admin(&self) -> PathBuf {
        self.file("admin.pub")
    }

    fn cacert(&self) -> PathBuf {
        self.file("root.crt")
    }

    fn cakey(&self) -> PathBuf {
        self.file("root.key")
    }
}

fn ls() {
    let mut fields: Vec<Field> = Field::all().collect();
    fields.sort_by_key(|f| f.modified);
    print_table(&fields, "fields", &["FIELD"], |f| vec![&f.name]);
}

fn new(name: String, key: Option<PathBuf>) {
    let field = Field::new(&name);
    let key = key.unwrap_or(home_ssh().join("id_ed25519.pub"));
    let pw = field.create(&key);
    println!("{} root.key password: {}", &name, &pw);
}

struct Image {
    name: String,
    version: String,
}

impl Image {
    fn new(name: &str, version: &str) -> Self {
        Self { name: name.to_string(), version: version.to_string() }
    }

    fn from_dir(entry: &fs::DirEntry) -> Option<Self> {
        Self::from_metadata(&entry.metadata().unwrap(), &entry.file_name())
    }

    fn from_path(path: &PathBuf) -> Option<Self> {
        match path.metadata() {
            Ok(m)  => Self::from_metadata(&m, &path.file_name().unwrap()),
            Err(_) => None,
        }
    }

    fn from_metadata(metadata: &fs::Metadata, file_name: &ffi::OsStr) -> Option<Self> {
        let file_name = file_name.to_str().unwrap();
        // ~/.barley/images/name_version.tar.zst
        if metadata.is_file() && file_name.ends_with(".tar.zst") {
            let mut parts = file_name.strip_suffix(".tar.zst").unwrap().splitn(2, '_');
            let name = parts.next().unwrap();
            let version = parts.next().unwrap_or("");
            Some(Self::new(name, version))
        } else {
            None
        }
    }

    fn latest(name: &str) -> Option<Self> {
        Self::all()
            .filter(|i| i.name == name)
            .max_by(|a, b| Self::compare_versions(&a.version, &b.version))
    }

    fn generate_version(&self) -> String {
        let version = Local::now().format("%Y%m%d").to_string();
        if let None = Self::from_path(&self.path()) {
            return version;
        }
        let mut base = Self::all()
            .filter(|i| i.name == self.name && i.version.starts_with(&version))
            .map(|i| i.version).max().unwrap();
        let mut subversion = 0;
        if *base != version {
            let parts: Vec<&str> = base.rsplitn(2, '.').collect();
            if parts.len() > 1 {
                if let Ok(i) = parts[0].parse() {
                    subversion = i;
                    base = parts[1].to_string();
                }
            }
        }
        format!("{}.{}", base, subversion + 1)
    }

    fn compare_versions(a: &str, b: &str) -> Ordering {
        match VersionCompare::compare(a, b).unwrap() {
            CompOp::Lt => Ordering::Less,
            CompOp::Gt => Ordering::Greater,
            CompOp::Eq => Ordering::Equal,
            _          => Ordering::Equal,
        }
    }

    fn all() -> impl Iterator<Item=Image> {
        fs::read_dir(&images_home()).unwrap()
            .filter_map(|entry| Self::from_dir(&entry.unwrap()))
    }

    fn path(&self) -> PathBuf {
        images_home().join(format!("{}_{}.tar.zst", &self.name, &self.version))
    }
}

fn ls_images() {
    let mut images: Vec<Image> = Image::all().collect();
    images.sort_by(|a, b| a.name.cmp(&b.name).then(a.version.cmp(&b.version)));
    print_table(&images, "images", &["IMAGE", "VERSION"], |i| vec![&i.name, &i.version]);
}

fn import(path: PathBuf) {
    match Image::from_path(&path) {
        Some(mut image) => {
            if image.version.is_empty() {
                image.version = image.generate_version();
            }
            if let Some(_) = Image::from_path(&image.path()) {
                panic!("Image version {} already exists", &image.version);
            }
            if let Err(err) = fs::hard_link(&path, &image.path()) {
                eprintln!("Failed to create hard link at {:?}: {:?}", &image.path(), err);
                fs::copy(&path, &image.path()).unwrap();
            }
        },
        None => panic!("{:?} is not a valid image file", path),
    }
}

fn exec(command: &mut Command) {
    let status = command.status().unwrap();
    if !status.success() {
        panic!("{:?} failed: {:?}", command, status);
    }
}

fn cat(src: &PathBuf) -> ChildStdout {
    let c = Command::new("/bin/cat")
        .arg(src)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    c.stdout.unwrap()
}

struct Machine {
    name: String,
    image: Image,
    field: Field,
    seed: Option<String>,
    data: Data,
}

impl Machine {
    fn new(
        image: String,
        version: Option<String>,
        field: Option<String>,
        seed: Option<String>,
        local: bool,
    ) -> Machine {
        if !local && seed.is_none() {
            // TODO: pick a seed with a scheduler
            panic!("Unable to pick a Seed for this machine, use --seed <host> or --local.");
        }
        let image = match version {
            Some(v) => Image::new(&image, &v),
            None    => Image::latest(&image).expect(&format!(
                "No images found for '{}'. Run 'sow import <path>'.",
                image,
            )),
        };
        let field = match field {
            Some(f) => Field::new(&f),
            None    => Field::latest().expect("No Barley fields found. Run 'sow new <name>'."),
        };
        let name = image.name.to_string();  // TODO: add NameCounter
        let data = Data::new(field.file(&name)).expect(&format!(
            "Failed to create data directory for machine '{}', does field '{}' exist?",
            &name, &field.name
        ));
        Machine { name, image, field, seed, data }
    }

    fn command(&self, script: &str) -> Command {
        match &self.seed {
            Some(seed) => {
                println!("Running ssh {} '{}'", &seed, &script);
                let mut c = Command::new("/usr/bin/ssh");
                c.arg(&seed).arg(script);
                c
            },
            None => {
                println!("Running sudo sh -c '{}'", &script);
                let mut c = Command::new("/usr/bin/sudo");
                c.arg("/bin/sh").arg("-c").arg(script);
                c
            },
        }
    }

    fn nspawn(&self, script: &str) -> Command {
        self.command(&format!("systemd-nspawn -M {} -UPq sh -c '{}'", self.name, script))
    }

    fn install(&self, from: &PathBuf, to: &str, mode: &str) {
        let to = format!("/var/lib/barley/{}", to);
        exec(
            self.nspawn(&format!("install -m {} -o barley -g barley /dev/null {} && \
                                  cat > {}", mode, to, to))
                .stdin(cat(from))
        )
    }

    fn import(&self) {
        exec(
            self.command(&format!("zstdcat | machinectl -q import-tar - {}", self.name))
                .stdin(cat(&self.image.path()))
        )
    }

    fn write_config(&self,  network: Option<Vec<String>>) -> Result<(), Error> {
        let mut cat = self.command(&format!(
            "mkdir -p /etc/systemd/nspawn && \
             cat > /etc/systemd/nspawn/{}.nspawn", self.name))
            .stdin(Stdio::piped())
            .spawn()?.stdin.unwrap();
        cat.write(b"[Network]\n").unwrap();
        if let Some(n) = network {
            for line in n {
                cat.write(line.as_bytes())?;
                cat.write(b"\n")?;
            }
        } else {
            cat.write(b"Bridge=br0\n")?;
        }
        Ok(())
    }

    fn start(&self, ca: bool, network: Option<Vec<String>>) {
        self.import();
        if ca {
            let key = self.data.file("machine.key");
            let cert = self.data.file("machine.crt");
            tls::generate(&key).unwrap();
            println!("When prompted, enter root.key password for {}", &self.field.name);
            tls::sign_ca(
                &self.name,
                &self.field.cacert(),
                &self.field.cakey(),
                &key,
                &cert,
            ).unwrap();
            self.install(&key, "machine.key", "600");
            self.install(&cert, "machine.crt", "644");
            self.install(&self.field.cacert(), "root.crt", "644");
            self.install(&self.field.admin(), "admin.pub", "644");
        }
        self.write_config(network).unwrap();
        exec(&mut self.command(&format!("machinectl start {}", self.name)))
    }
}

/// Ephemeral bare-metal provisioning system
#[derive(StructOpt)]
struct Opt {
    /// Specify the Barley field (default is the most recently modified field)
    #[structopt(short, long, env = "BARLEY_FIELD")]
    field: Option<String>,

    #[structopt(subcommand)]
    op: Option<Op>
}

#[derive(StructOpt)]
enum Op {
    /// List Barley fields
    Fields,

    /// Initialize a new Barley field
    New {
        /// Name of the new Barley field to be created
        name: String,
        /// SSH public key that will be granted root access to Seeds,
        /// default: ~/.ssh/id_ed25519.pub
        #[structopt(short, long)]
        key: Option<PathBuf>,
    },

    /// List imported images
    Images,

    /// Import an image
    Import {
        /// Path to the image file (name.tar.zst or name_version.tar.zst)
        #[structopt(parse(from_os_str))]
        path: PathBuf,
    },

    /// Start a new machine from an imported image
    Start {
        /// Image name
        image: String,

        /// Image version, default: latest version
        #[structopt(short, long)]
        version: Option<String>,

        /// Specify the Seed host to start machines on
        #[structopt(short, long, required_if("local", "false"))]
        seed: Option<String>,

        /// Start machines locally rather than on a remote Seed host
        #[structopt(long, conflicts_with("seed"))]
        local: bool,

        /// Install a Certificate Authority key on the machine
        #[structopt(long)]
        ca: bool,

        /// Network options for systemd.nspawn(5), default: Bridge=br0
        #[structopt(short, long, number_of_values = 1)]
        network: Option<Vec<String>>,
    },
}

fn main() {
    let opt = Opt::from_args();
    setup();
    match opt.op {
        None => { ls() },
        Some(Op::Fields) => { ls() },
        Some(Op::New { name, key }) => { new(name, key) },
        Some(Op::Images) => { ls_images() },
        Some(Op::Import { path }) => { import(path) },
        Some(Op::Start { image, version, seed, local, ca, network }) => {
            Machine::new(image, version, opt.field, seed, local).start(ca, network)
        },
    };
}
