#![allow(unused_imports)]
#![deny(
    non_ascii_idents,
    non_shorthand_field_patterns,
    no_mangle_generic_items,
    overflowing_literals,
    path_statements,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unused_must_use,
    clippy::unwrap_used
)]

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;

use actix_cors::Cors;
use actix_web::middleware::{Logger, NormalizePath, TrailingSlash};
use actix_web::{App, Error, HttpServer};
use paperclip::actix::{
    api_v2_operation,
    web::{self, Json},
    OpenApiExt,
};
use paperclip::v2::models::{DefaultApiRaw, Info, Tag};
use serde_json::Value;
use sqlx::postgres::PgPoolOptions;

use crate::router::routes;
use crate::service::auth;

mod error;
mod handler;
mod model;
mod router;
mod service;
#[cfg(test)]
mod tests;
mod types;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    pretty_env_logger::init();
    let jwt_secret = dotenv::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let database_url = dotenv::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let storage_path = dotenv::var("STORAGE_PATH").expect("STORAGE_PATH must be set");
    let port = dotenv::var("PORT").expect("PORT must be set");
    let workers = dotenv::var("WORKERS")
        .expect("WORKERS must be set")
        .parse()
        .expect("Invalid WORKERS");
    let pool_size = dotenv::var("DB_POOL")
        .expect("DB_POOL must be set")
        .parse()
        .expect("Invalid DB_POOL");
    let pool = PgPoolOptions::new()
        .max_connections(pool_size)
        .connect(&database_url)
        .await
        .expect("Unable to connect to DB");

    let auth_service = auth::AuthService { secret: jwt_secret };
    let spec_service = service::spec::spec::SpecService { pool: pool.clone() };
    let meet_service = service::meet::MeetService { pool: pool.clone() };
    let image_service = service::image::FileService { storage_path };

    HttpServer::new(move || {
        let spec = DefaultApiRaw {
            info: Info {
                version: "0.1".into(),
                title: "Explany backend".into(),
                ..Default::default()
            },
            ..Default::default()
        };
        let cors = Cors::permissive();
        App::new()
            .wrap_api_with_spec(spec)
            .configure(routes)
            .wrap(Logger::default())
            .wrap(NormalizePath::new(TrailingSlash::Trim))
            .wrap(cors)
            .wrap(Logger::new(r#"%a "%r" %s %b "%{Referer}i" "%{User-Agent}i" %T"#))
            .app_data(web::Data::new(auth_service.clone()))
            .app_data(web::Data::new(spec_service.clone()))
            .app_data(web::Data::new(image_service.clone()))
            .app_data(web::Data::new(meet_service.clone()))
            .with_json_spec_at("/swagger-spec")
            .with_swagger_ui_at("/swagger-ui")
            .build()
    })
    .bind(format!("127.0.0.1:{}", port))?
    .workers(workers)
    .run()
    .await
}
