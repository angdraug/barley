use actix_files::NamedFile;
use actix_web::{get, App, HttpResponse, HttpServer, middleware, post, Result, web};
use std::env;

use barley::{Error, Registration, Sower};

const DNSMASQ: &str = "/etc/dnsmasq.d/barley.conf";
const IMAGE_DIR: &str = "/srv/barley";
const DATA_DIR: &str = "/var/lib/barley";

#[get("/seed.vmlinuz")]
async fn vmlinuz(sower: web::Data<Sower>) -> Result<NamedFile> {
    Ok(NamedFile::open(sower.images.file("seed.vmlinuz"))?)
}

#[get("/seed.cpio.zst")]
async fn cpio(sower: web::Data<Sower>) -> Result<NamedFile> {
    Ok(NamedFile::open(sower.images.file("seed.cpio.zst"))?)
}

#[get("/seed.ipxe")]
async fn ipxe(sower: web::Data<Sower>) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().body(sower.ipxe()?))
}

#[get("/init/{name}")]
async fn init(
    sower:           web::Data<Sower>,
    web::Path(name): web::Path<String>,
) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().body(sower.init(&name)?))
}

#[post("/register/{name}")]
async fn register(
    sower:           web::Data<Sower>,
    web::Path(name): web::Path<String>,
    registration:    web::Json<Registration>
) -> Result<HttpResponse, Error> {
    match sower.register(&name, &registration) {
        Ok(certs) => Ok(HttpResponse::Ok().json(certs)),
        Err(err) => {
            eprintln!("{:?}", err);
            Err(err)
        },
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env::set_var("RUST_LOG", "actix_web=debug,actix_server=info");
    env_logger::init();

    let sower = Sower::new(DNSMASQ, IMAGE_DIR, DATA_DIR);
    let binding = sower.binding();

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .data(sower.clone())
            .service(vmlinuz)
            .service(cpio)
            .service(ipxe)
            .service(init)
            .service(register)
    })
    .bind(binding)?
    .run()
    .await
}
