use actix_session::Session;
use actix_web::{
    delete, dev::HttpServiceFactory, get, post, put, web, HttpRequest, HttpResponse, Responder,
};
use serde::{Deserialize, Serialize};

use crate::{
    model::{
        self,
        tasks::Update,
        types::{TaskPriority, TaskState, Todo, TodoReq},
    },
    utils::{binary_to_ulid, check_is_logged_in, ulid_to_binary},
};

pub fn tasks_router() -> impl HttpServiceFactory {
    web::scope("/tasks")
        .service(get_tasks_me)
        .service(post_task)
        .service(get_task)
        .service(delete_task)
        .service(put_task)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResponse {
    pub id: String,
    pub author_id: String,
    pub title: String,
    pub description: String,
    pub created_at: String,
    pub updated_at: String,

    pub state: TaskState,
    pub priority: Option<TaskPriority>,
    pub due_date: Option<String>,
}
impl TryFrom<Todo> for TaskResponse {
    type Error = anyhow::Error;

    fn try_from(value: Todo) -> Result<Self, Self::Error> {
        let id = binary_to_ulid(value.id.as_slice())?;
        let author_id_content = value
            .author_id
            .ok_or_else(|| anyhow::anyhow!("Invalid author_id"))?;
        let author_id = binary_to_ulid(author_id_content.as_slice())?;
        let created_at = value.created_at.format("%Y-%m-%d %H:%M:%S").to_string();
        let updated_at = value.updated_at.format("%Y-%m-%d %H:%M:%S").to_string();
        let due_date = value
            .due_date
            .map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string());

        Ok(Self {
            id: id.to_string(),
            author_id: author_id.to_string(),
            title: value.title,
            description: value.description,
            created_at,
            updated_at,

            state: value.state,
            priority: value.priority,
            due_date,
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetTaskQuery {
    limit: Option<usize>,
    offset: Option<usize>,

    state: Option<TaskState>,
    priority: Option<TaskPriority>,
}
#[get("/me")]
pub async fn get_tasks_me(
    session: Session,
    pool: web::Data<sqlx::MySqlPool>,
    query: web::Query<GetTaskQuery>,
) -> impl Responder {
    async fn get_tasks_me_inner(
        session: Session,
        pool: web::Data<sqlx::MySqlPool>,
        query: web::Query<GetTaskQuery>,
    ) -> Result<HttpResponse, HttpResponse> {
        let user_ulid = check_is_logged_in(session, pool.as_ref())
            .await
            .map_err(|e| HttpResponse::Unauthorized().body(format!("Unauthorized: {}", e)))?;

        let limit = match (query.limit, query.offset) {
            (Some(limit), Some(offset)) => Some(model::tasks::Limit::LimitOffset(limit, offset)),
            (Some(limit), None) => Some(model::tasks::Limit::Limit(limit)),
            (None, Some(_)) => {
                return Err(HttpResponse::BadRequest().body("Invalid query"));
            }
            (None, None) => None,
        };

        let tasks = model::tasks::get_tasks(pool.as_ref(), user_ulid, limit, None)
            .await
            .map_err(|e| {
                HttpResponse::InternalServerError().body(format!("Internal Server Error: {}", e))
            })?
            .into_iter()
            .map(TaskResponse::try_from)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| {
                HttpResponse::InternalServerError().body(format!("Internal Server Error: {}", e))
            })?;

        Ok(HttpResponse::Ok().json(tasks))
    }

    get_tasks_me_inner(session, pool, query)
        .await
        .unwrap_or_else(std::convert::identity)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostTaskRequest {
    pub title: String,
    pub description: String,
    pub state: TaskState,
    pub priority: Option<TaskPriority>,
    pub due_date: Option<String>,
}

#[post("/")]
pub async fn post_task(
    _req: HttpRequest,
    body: web::Json<PostTaskRequest>,
    session: Session,
    pool: web::Data<sqlx::MySqlPool>,
) -> impl Responder {
    async fn post_task_inner(
        _req: HttpRequest,
        body: web::Json<PostTaskRequest>,
        session: Session,
        pool: web::Data<sqlx::MySqlPool>,
    ) -> Result<HttpResponse, HttpResponse> {
        let user_ulid = check_is_logged_in(session, pool.as_ref())
            .await
            .map_err(|e| HttpResponse::Unauthorized().body(format!("Unauthorized: {}", e)))?;

        let task_ulid = ulid::Ulid::new();

        model::tasks::insert_task(
            pool.as_ref(),
            TodoReq {
                id: ulid_to_binary(task_ulid).to_vec(),
                author_id: Some(ulid_to_binary(user_ulid).to_vec()),
                title: body.title.clone(),
                description: body.description.clone(),
                state: body.state,
                priority: body.priority,
                due_date: body.due_date.as_ref().map(|d| {
                    chrono::NaiveDateTime::parse_from_str(d, "%Y-%m-%d %H:%M:%S")
                        .expect("Invalid due_date")
                }),
            },
        )
        .await
        .map_err(|e| {
            HttpResponse::InternalServerError().body(format!("Internal Server Error: {}", e))
        })?;

        Ok(HttpResponse::Created().finish())
    }

    post_task_inner(_req, body, session, pool)
        .await
        .unwrap_or_else(std::convert::identity)
}

#[get("/{id}")]
pub async fn get_task(
    _req: HttpRequest,
    id: web::Path<String>,
    session: Session,
    pool: web::Data<sqlx::MySqlPool>,
) -> impl Responder {
    async fn get_task_inner(
        _req: HttpRequest,
        id: web::Path<String>,
        session: Session,
        pool: web::Data<sqlx::MySqlPool>,
    ) -> Result<HttpResponse, HttpResponse> {
        let user_ulid = check_is_logged_in(session, pool.as_ref())
            .await
            .map_err(|e| HttpResponse::Unauthorized().body(format!("Unauthorized: {}", e)))?;

        let task_ulid = ulid::Ulid::from_string(&id)
            .map_err(|e| HttpResponse::BadRequest().body(format!("Invalid task id: {}", e)))?;

        let task = model::tasks::get_task(pool.as_ref(), task_ulid)
            .await
            .map_err(|e| {
                HttpResponse::InternalServerError().body(format!("Internal Server Error: {}", e))
            })?
            .ok_or_else(|| HttpResponse::NotFound().body("Not Found"))?;

        if task.author_id != Some(ulid_to_binary(user_ulid).to_vec()) {
            return Err(HttpResponse::Forbidden().body("Forbidden"));
        }

        Ok(
            HttpResponse::Ok().json(TaskResponse::try_from(task).map_err(|e| {
                HttpResponse::InternalServerError().body(format!("Internal Server Error: {}", e))
            })?),
        )
    }

    get_task_inner(_req, id, session, pool)
        .await
        .unwrap_or_else(std::convert::identity)
}

#[delete("/{id}")]
pub async fn delete_task(
    _req: HttpRequest,
    id: web::Path<String>,
    session: Session,
    pool: web::Data<sqlx::MySqlPool>,
) -> impl Responder {
    async fn delete_task_inner(
        _req: HttpRequest,
        id: web::Path<String>,
        session: Session,
        pool: web::Data<sqlx::MySqlPool>,
    ) -> Result<HttpResponse, HttpResponse> {
        let mut tx = pool.begin().await.map_err(|e| {
            HttpResponse::InternalServerError().body(format!("Internal Server Error: {}", e))
        })?;

        let user_ulid = check_is_logged_in(session, &mut tx)
            .await
            .map_err(|e| HttpResponse::Unauthorized().body(format!("Unauthorized: {}", e)))?;

        let task_ulid = ulid::Ulid::from_string(&id)
            .map_err(|e| HttpResponse::BadRequest().body(format!("Invalid task id: {}", e)))?;

        let task = model::tasks::get_task_with_lock(&mut tx, task_ulid)
            .await
            .map_err(|e| {
                HttpResponse::InternalServerError().body(format!("Internal Server Error: {}", e))
            })?
            .ok_or_else(|| HttpResponse::NotFound().body("Not Found"))?;

        if task.author_id != Some(ulid_to_binary(user_ulid).to_vec()) {
            return Err(HttpResponse::Forbidden().body("Forbidden"));
        }

        model::tasks::delete_task(&mut tx, task_ulid)
            .await
            .map_err(|e| {
                HttpResponse::InternalServerError().body(format!("Internal Server Error: {}", e))
            })?;

        tx.commit().await.map_err(|e| {
            HttpResponse::InternalServerError().body(format!("Internal Server Error: {}", e))
        })?;

        Ok(HttpResponse::NoContent().finish())
    }

    delete_task_inner(_req, id, session, pool)
        .await
        .unwrap_or_else(std::convert::identity)
}

#[derive(Debug, Clone, Deserialize)]
pub struct PutTaskRequest {
    #[serde(default)]
    pub title: Update<String>,
    #[serde(default)]
    pub description: Update<String>,
    #[serde(default)]
    pub state: Update<TaskState>,
    #[serde(default)]
    pub priority: Update<Option<TaskPriority>>,
    #[serde(default)]
    pub due_date: Update<Option<String>>,
}
#[put("/{id}")]
pub async fn put_task(
    _req: HttpRequest,
    id: web::Path<String>,
    body: web::Json<PutTaskRequest>,
    session: Session,
    pool: web::Data<sqlx::MySqlPool>,
) -> impl Responder {
    async fn put_task_inner(
        _req: HttpRequest,
        id: web::Path<String>,
        body: web::Json<PutTaskRequest>,
        session: Session,
        pool: web::Data<sqlx::MySqlPool>,
    ) -> Result<HttpResponse, HttpResponse> {
        let mut tx = pool.begin().await.map_err(|e| {
            HttpResponse::InternalServerError().body(format!("Internal Server Error: {}", e))
        })?;

        let user_ulid = check_is_logged_in(session, &mut tx)
            .await
            .map_err(|e| HttpResponse::Unauthorized().body(format!("Unauthorized: {}", e)))?;

        let task_ulid = ulid::Ulid::from_string(&id)
            .map_err(|e| HttpResponse::BadRequest().body(format!("Invalid task id: {}", e)))?;

        let task = model::tasks::get_task_with_lock(&mut tx, task_ulid)
            .await
            .map_err(|e| {
                HttpResponse::InternalServerError().body(format!("Internal Server Error: {}", e))
            })?
            .ok_or_else(|| HttpResponse::NotFound().body("Not Found"))?;

        if task.author_id != Some(ulid_to_binary(user_ulid).to_vec()) {
            return Err(HttpResponse::Forbidden().body("Forbidden"));
        }

        let task_req = model::tasks::UpdateTask {
            title: body.title.clone(),
            description: body.description.clone(),
            state: body.state.clone(),
            priority: body.priority.clone(),
            due_date: body
                .due_date
                .clone()
                .map(|d| {
                    d.map(|s| {
                        chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%d").map_err(|e| {
                            HttpResponse::BadRequest().body(format!("Invalid due date: {}", e))
                        })
                    })
                    .transpose()
                })
                .transpose()?,
        };

        model::tasks::update_task(&mut tx, task_ulid, task_req)
            .await
            .map_err(|e| {
                HttpResponse::InternalServerError().body(format!("Internal Server Error: {}", e))
            })?;

        tx.commit().await.map_err(|e| {
            HttpResponse::InternalServerError().body(format!("Internal Server Error: {}", e))
        })?;

        Ok(HttpResponse::NoContent().finish())
    }

    put_task_inner(_req, id, body, session, pool)
        .await
        .unwrap_or_else(std::convert::identity)
}
