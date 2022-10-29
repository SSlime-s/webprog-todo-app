use std::env;

use actix_web::{get, App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use once_cell::sync::OnceCell;
use sqlx::{mysql::MySqlPoolOptions, MySqlPool};

pub static POOL: OnceCell<MySqlPool> = OnceCell::new();

#[get("/")]
async fn hello_world() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let hostname = env::var("MARIADB_HOSTNAME").unwrap();
    let database = env::var("MARIADB_DATABASE").unwrap();
    let username = env::var("MARIADB_USERNAME").unwrap();
    let password = env::var("MARIADB_PASSWORD").unwrap();

    let pool = MySqlPoolOptions::new()
        .max_connections(10)
        .connect(&format!(
            "mysql://{}:{}@{}/{}",
            username, password, hostname, database
        ))
        .await
        .unwrap();
    POOL.set(pool).unwrap();

    let query = "SHOW TABLES;";
    let tables = sqlx::query(query)
        .fetch_all(&*POOL.get().unwrap())
        .await
        .unwrap();
    println!("Tables: {:?}", tables);

    HttpServer::new(|| App::new().service(hello_world))
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}
