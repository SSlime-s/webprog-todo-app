mod model;
mod router;
mod utils;

use std::env;

use actix_session::{storage::CookieSessionStore, Session, SessionMiddleware};
use actix_web::{cookie::Key, get, web::Data, App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use once_cell::sync::OnceCell;
use sqlx::{mysql::MySqlPoolOptions, MySqlPool};

use crate::router::account::{login, signup, logout, delete_user};

pub static POOL: OnceCell<MySqlPool> = OnceCell::new();

#[get("/")]
async fn hello_world(session: Session) -> impl Responder {
    let user_id = session.get::<String>("user_id").unwrap();
    log::info!("user_id: {:?}", user_id);
    if let Some(user_id) = user_id {
        HttpResponse::Ok().body(format!("Hello, {}!", user_id))
    } else {
        HttpResponse::Ok().body("Hello, world!")
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env::set_var("RUST_LOG", "info");
    dotenv().ok();
    env_logger::init();

    let hostname = env::var("MARIADB_HOSTNAME").unwrap();
    let database = env::var("MARIADB_DATABASE").unwrap();
    let username = env::var("MARIADB_USERNAME").unwrap();
    let password = env::var("MARIADB_PASSWORD").unwrap();

    let secret_key = env::var("SECRET_KEY").unwrap();
    let secret_key = Key::from(secret_key.as_bytes());

    let pool = MySqlPoolOptions::new()
        .max_connections(10)
        .connect(&format!(
            "mysql://{}:{}@{}/{}",
            username, password, hostname, database
        ))
        .await
        .unwrap();
    // POOL.set(pool).unwrap();

    let query = "SHOW TABLES;";
    let tables = sqlx::query(query).fetch_all(&pool).await.unwrap();
    println!("Tables: {:?}", tables);

    HttpServer::new(move || {
        App::new()
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), secret_key.clone())
                    .cookie_secure(false)
                    .build(),
            )
            .app_data(Data::new(pool.clone()))
            .service(hello_world)
            .service(signup)
            .service(login)
            .service(logout)
            .service(delete_user)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
