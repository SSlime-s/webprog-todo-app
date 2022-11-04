use actix_session::Session;
use actix_web::{dev::HttpServiceFactory, get, post, web, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

use crate::{
    model::{
        self,
        types::{TaskPriority, TaskState, Todo, TodoReq},
    },
    utils::{binary_to_ulid, check_is_logged_in, ulid_to_binary},
};

pub fn tasks_router() -> impl HttpServiceFactory {
    web::scope("/tasks")
        .service(get_tasks_me)
        .service(post_task)
    // .service(get_task)
    // .service(delete_task)
    // .service(put_task)
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

#[get("/me")]
pub async fn get_tasks_me(session: Session, pool: web::Data<sqlx::MySqlPool>) -> impl Responder {
    let user_ulid = check_is_logged_in(session, pool.as_ref()).await;
    if let Err(e) = user_ulid {
        return HttpResponse::Unauthorized().body(format!("Unauthorized: {}", e));
    }
    let user_ulid = user_ulid.unwrap();

    let tasks = model::tasks::get_tasks(pool.as_ref(), user_ulid, None, None).await;
    if let Err(e) = tasks {
        return HttpResponse::InternalServerError().body(format!("Internal Server Error: {}", e));
    }
    let tasks = tasks.unwrap();

    let tasks = tasks
        .into_iter()
        .map(TaskResponse::try_from)
        .fold(Ok(Vec::new()), |acc, x| {
            if let Err(e) = x {
                return Err(e);
            }
            let mut acc = acc?;
            acc.push(x.unwrap());
            Ok(acc)
        });
    if let Err(e) = tasks {
        return HttpResponse::InternalServerError().body(format!("Internal Server Error: {}", e));
    }

    HttpResponse::Ok().json(tasks.unwrap())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostTaskRequest {
    pub title: String,
    pub description: String,
    pub state: TaskState,
    pub priority: TaskPriority,
    pub due_date: Option<String>,
}

#[post("/")]
pub async fn post_task(
    _req: HttpRequest,
    body: web::Json<PostTaskRequest>,
    session: Session,
    pool: web::Data<sqlx::MySqlPool>,
) -> impl Responder {
    let user_ulid = check_is_logged_in(session, pool.as_ref()).await;
    if let Err(e) = user_ulid {
        return HttpResponse::Unauthorized().body(format!("Unauthorized: {}", e));
    }
    let user_ulid = user_ulid.unwrap();
    let task_ulid = ulid::Ulid::new();

    let result = model::tasks::insert_task(
        pool.as_ref(),
        TodoReq {
            id: ulid_to_binary(task_ulid).to_vec(),
            author_id: Some(ulid_to_binary(user_ulid).to_vec()),
            title: body.title.clone(),
            description: body.description.clone(),
            state: body.state,
            priority: Some(body.priority),
            due_date: body.due_date.as_ref().map(|d| {
                chrono::NaiveDateTime::parse_from_str(d, "%Y-%m-%d %H:%M:%S")
                    .expect("Invalid due_date")
            }),
        },
    )
    .await;
    if let Err(e) = result {
        return HttpResponse::InternalServerError().body(format!("Internal Server Error: {}", e));
    }

    HttpResponse::Created().finish()
}