mod routes;
use crate::routes::{upload_csv, upload_img};
use actix_cors::Cors;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder, middleware::Logger};
use env_logger::Env;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    HttpServer::new(|| {
        App::new()
            .wrap(Cors::permissive())
            .wrap(Logger::default())
            .service(hello)
            .route("/decodeIMG", web::post().to(upload_img))
            .route("/decodeCSV", web::post().to(upload_csv))
        //.service(upload)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
