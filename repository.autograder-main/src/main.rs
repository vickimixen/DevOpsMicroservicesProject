#![feature(decl_macro, proc_macro_hygiene)]

extern crate base64;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
extern crate dotenv;
extern crate jsonwebtoken;
#[macro_use(error)]
extern crate log;
extern crate r2d2;
extern crate reqwest;
#[macro_use]
extern crate rocket;
extern crate rocket_contrib;
extern crate rocket_cors;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use dotenv::dotenv;
use rocket::http::Method;
use rocket_cors::{AllowedHeaders, AllowedOrigins, Cors, CorsOptions};

mod assignments;
mod auth;
mod config;
mod connection;
mod files;
mod schema;
mod submissions;

fn make_cors() -> Cors {
    CorsOptions {
        allowed_origins: AllowedOrigins::All,
        allowed_methods: vec![Method::Options, Method::Get, Method::Post, Method::Patch]
            .into_iter()
            .map(From::from)
            .collect(),
        allowed_headers: AllowedHeaders::All,
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors()
    .expect("error while building CORS")
}

fn main() {
    dotenv().ok();
    let mut rkt = rocket::ignite().manage(connection::init_pool());
    rkt = submissions::router::create_routes(rkt);
    rkt = files::router::create_routes(rkt);
    rkt = assignments::router::create_routes(rkt);
    rkt.attach(make_cors())
        .attach(config::AppState::manage())
        .launch();
}
