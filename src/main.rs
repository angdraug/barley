use actix_files::NamedFile;
use actix_web::{get, App, HttpResponse, HttpServer, middleware, post, Responder, Result, web};
use serde::Deserialize;
use std::{env, net};

use barley;

#[get("/seed.vmlinuz")]
async fn vmlinuz(sower: web::Data<barley::Sower<'_>>) -> Result<NamedFile> {
    Ok(NamedFile::open(sower.image("seed.vmlinuz"))?)
}

#[get("/seed.cpio.gz")]
async fn cpio(sower: web::Data<barley::Sower<'_>>) -> Result<NamedFile> {
    Ok(NamedFile::open(sower.image("seed.cpio.gz"))?)
}

#[get("/seed.ipxe")]
async fn ipxe() -> impl Responder {
    HttpResponse::Ok().body(barley::Seed::reserve().ipxe())
}

#[get("/init/{name}")]
async fn init(
    sower:           web::Data<barley::Sower<'_>>,
    web::Path(name): web::Path<String>,
) -> impl Responder {
    HttpResponse::Ok().body(barley::Seed::new(name).init(sower.ip))
}

#[derive(Deserialize)]
struct RegisterForm {
    otp: String,
    ip:  net::IpAddr,
    ssh: String,
}

#[post("/register/{name}")]
async fn register(
    web::Path(name): web::Path<String>,
    form:            web::Json<RegisterForm>
) -> impl Responder {
    match barley::Seed::new(name).register(&form.otp, &form.ip, &form.ssh) {
        Ok(body) => HttpResponse::Ok().body(body),
        Err(err) => {
            eprintln!("{:?}", err);
            HttpResponse::BadRequest().body(format!("{:#?}", err))
        },
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env::set_var("RUST_LOG", "actix_web=debug,actix_server=info");
    env_logger::init();

    let sower = barley::Sower::new();
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
