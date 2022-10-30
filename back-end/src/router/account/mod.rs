use actix_session::Session;
use actix_web::{delete, post, web, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

use crate::{
    model::users::{get_user_from_username, insert_user, is_username_exists, remove_user},
    utils::{binary_to_ulid, ulid_to_binary},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignupRequest {
    pub username: String,
    pub display_name: String,
    pub password: String,
}

#[post("/signup")]
pub async fn signup(
    _req: HttpRequest,
    body: web::Json<SignupRequest>,
    session: Session,
    pool: web::Data<sqlx::MySqlPool>,
) -> impl Responder {
    let content = body.0;
    let mut tx = pool.begin().await.unwrap();
    if content.username.is_empty() || content.password.is_empty() {
        return HttpResponse::BadRequest().body("Invalid username or password");
    }

    let is_username_exists = is_username_exists(&mut tx, &content.username).await;
    if is_username_exists.is_err() {
        return HttpResponse::InternalServerError().body("Internal server error");
    }
    if is_username_exists.unwrap() {
        return HttpResponse::BadRequest().body("Username already exists");
    }

    let id = ulid::Ulid::new();

    let hashed_password =
        bcrypt::hash_with_salt(content.password, bcrypt::DEFAULT_COST, ulid_to_binary(id))
            .map(|s| s.to_string());
    if let Err(e) = hashed_password {
        return HttpResponse::InternalServerError().body(format!("Internal Server Error: {}", e));
    }
    let hashed_password = hashed_password.unwrap();

    if let Err(e) = insert_user(
        &mut tx,
        Some(id),
        &content.username,
        &content.display_name,
        hashed_password.as_bytes(),
    )
    .await
    {
        return HttpResponse::InternalServerError().body(format!("Internal Server Error: {}", e));
    }

    if let Err(e) = session.insert("user_id", id.to_string()) {
        return HttpResponse::InternalServerError().body(format!("Internal Server Error: {}", e));
    }
    tx.commit().await.unwrap();

    HttpResponse::Created().finish()
}

#[post("/login")]
pub async fn login(
    _req: HttpRequest,
    body: web::Json<SignupRequest>,
    session: Session,
    pool: web::Data<sqlx::MySqlPool>,
) -> impl Responder {
    let content = body.0;
    if content.username.is_empty() || content.password.is_empty() {
        return HttpResponse::BadRequest().body("Invalid username or password");
    }

    let user = get_user_from_username(pool.as_ref(), &content.username).await;
    if user.is_err() {
        return HttpResponse::InternalServerError().body("Internal server error");
    }
    let user = user.unwrap();
    if user.is_none() {
        return HttpResponse::BadRequest()
            .body(format!("Username {} does not exist", content.username));
    }
    let user = user.unwrap();

    let is_valid = bcrypt::verify(
        &content.password,
        std::str::from_utf8(&user.hashed_password).unwrap(),
    );
    if is_valid.is_err() {
        return HttpResponse::InternalServerError().body("Internal server error");
    }
    if !is_valid.unwrap() {
        return HttpResponse::BadRequest().body("Invalid password");
    }

    if let Err(e) = session.insert("user_id", binary_to_ulid(&user.id).unwrap().to_string()) {
        return HttpResponse::InternalServerError().body(format!("Internal Server Error: {}", e));
    }

    HttpResponse::NoContent().finish()
}

#[delete("/logout")]
pub async fn logout(session: Session) -> impl Responder {
    session.purge();
    HttpResponse::NoContent().finish()
}

#[delete("/user")]
pub async fn delete_user(
    _req: HttpRequest,
    session: Session,
    pool: web::Data<sqlx::MySqlPool>,
) -> impl Responder {
    let user_id = session.get::<String>("user_id");
    if user_id.is_err() {
        return HttpResponse::InternalServerError().body("Internal server error");
    }
    let user_id = user_id.unwrap();
    if user_id.is_none() {
        return HttpResponse::BadRequest().body("Not logged in");
    }
    let user_id = user_id.unwrap();
    let user_ulid = ulid::Ulid::from_string(&user_id);
    if user_ulid.is_err() {
        return HttpResponse::InternalServerError().body("Internal server error");
    }
    let user_ulid = user_ulid.unwrap();

    if let Err(e) = remove_user(pool.as_ref(), user_ulid).await {
        return HttpResponse::InternalServerError().body(format!("Internal Server Error: {}", e));
    }

    session.purge();
    HttpResponse::NoContent().finish()
}
