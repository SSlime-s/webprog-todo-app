use actix_session::Session;
use actix_web::{
    delete, dev::HttpServiceFactory, get, patch, post, web, HttpRequest, HttpResponse,
    Responder,
};
use serde::{Deserialize, Serialize};

use crate::{
    model::{
        self,
        users::{get_user, get_user_from_username, insert_user, is_username_exists, remove_user},
        Update,
    },
    utils::{binary_to_ulid, check_is_logged_in, ulid_to_binary},
};

pub fn account_router() -> impl HttpServiceFactory {
    web::scope("")
        .service(post_signup)
        .service(post_login)
        .service(delete_logout)
        .service(get_me)
        .service(delete_me)
        .service(patch_me)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignupRequest {
    pub username: String,
    pub display_name: String,
    pub password: String,
}

#[post("/signup")]
pub async fn post_signup(
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

    let hashed_password = hash_password(&content.password, &id);
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}
#[post("/login")]
pub async fn post_login(
    _req: HttpRequest,
    body: web::Json<LoginRequest>,
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
pub async fn delete_logout(session: Session) -> impl Responder {
    session.purge();
    HttpResponse::NoContent().finish()
}

#[derive(Serialize, Deserialize)]
pub struct MeResponse {
    pub id: String,
    pub username: String,
    pub display_name: String,
}

#[get("/me")]
pub async fn get_me(session: Session, pool: web::Data<sqlx::MySqlPool>) -> impl Responder {
    if let Some(user_id) = session.get::<String>("user_id").unwrap() {
        let id = ulid::Ulid::from_string(&user_id).unwrap();

        let user = get_user(pool.as_ref(), id).await;
        if user.is_err() {
            return HttpResponse::InternalServerError().body("Internal server error");
        }
        let user = user.unwrap();
        if user.is_none() {
            session.purge();
            return HttpResponse::BadRequest().body("User does not exist, session purged");
        }
        let user = user.unwrap();
        if user.username.is_none() {
            return HttpResponse::InternalServerError().body("Internal server error");
        }

        HttpResponse::Ok().json(MeResponse {
            id: user_id,
            username: user.username.unwrap(),
            display_name: user.display_name,
        })
    } else {
        HttpResponse::Unauthorized().finish()
    }
}

#[delete("/me")]
pub async fn delete_me(
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

#[derive(Debug, Clone, Deserialize)]
pub struct PutUserRequest {
    #[serde(default)]
    pub username: Update<String>,
    #[serde(default)]
    pub display_name: Update<String>,
    #[serde(default)]
    pub password: Update<String>,
}
#[patch("/me")]
pub async fn patch_me(
    _req: HttpRequest,
    body: web::Json<PutUserRequest>,
    session: Session,
    pool: web::Data<sqlx::MySqlPool>,
) -> impl Responder {
    let Ok(mut tx) = pool.begin().await else {
        return HttpResponse::InternalServerError().body("Internal server error");
    };

    let Ok(user_ulid) = check_is_logged_in(session, &mut tx).await else {
        return HttpResponse::Unauthorized().finish();
    };

    let password = body.password.clone();
    let Ok(hashed_password) = password.map(|password| {
        hash_password(&password, &user_ulid).map(|h| h.as_bytes().to_vec())
    }).transpose() else {
        return HttpResponse::InternalServerError().body("Internal server error");
    };

    let user_req = model::users::UpdateUser {
        username: body.username.clone(),
        display_name: body.display_name.clone(),
        hashed_password,
    };

    let Ok(_) = model::users::update_user(&mut tx, user_ulid, user_req).await else {
        return HttpResponse::InternalServerError().body("Internal server error");
    };

    let Ok(_) = tx.commit().await else {
        return HttpResponse::InternalServerError().body("Internal server error");
    };

    HttpResponse::NoContent().finish()
}

#[get("/available/{username}")]
pub async fn get_available(
    _req: HttpRequest,
    username: web::Path<String>,
    pool: web::Data<sqlx::MySqlPool>,
) -> impl Responder {
    let Ok(available) = model::users::is_username_exists(pool.as_ref(), &username).await else {
        return HttpResponse::InternalServerError().body("Internal server error");
    };

    HttpResponse::Ok().json(available)
}

pub fn hash_password(
    password: &str,
    user_ulid: &ulid::Ulid,
) -> Result<String, bcrypt::BcryptError> {
    bcrypt::hash_with_salt(password, bcrypt::DEFAULT_COST, ulid_to_binary(*user_ulid))
        .map(|s| s.to_string())
}
