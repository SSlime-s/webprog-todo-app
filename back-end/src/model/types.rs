use sqlx::{FromRow, Type};

#[derive(Debug, Clone, FromRow)]
pub struct Tag {
    pub id: Vec<u8>,
    pub name: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}
#[derive(Debug, Clone, FromRow)]
pub struct TagReq {
    pub id: Vec<u8>,
    pub name: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct User {
    pub id: Vec<u8>,
    pub username: String,
    pub display_name: String,
    pub hashed_password: Vec<u8>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    #[sqlx(default)]
    pub deleted_at: Option<chrono::NaiveDateTime>,
}
#[derive(Debug, Clone, FromRow)]
pub struct UserReq {
    pub id: Vec<u8>,
    pub username: String,
    pub display_name: String,
    pub hashed_password: Vec<u8>,
}

#[derive(Debug, Clone, Copy, Type)]
#[sqlx(rename_all = "kebab-case")]
pub enum TaskState {
    Icebox,
    Todo,
    InProgress,
    Done,
}

#[derive(Debug, Clone, Copy, Type)]
#[sqlx(rename_all = "kebab-case")]
pub enum TaskPriority {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, FromRow)]
pub struct Todo {
    pub id: Vec<u8>,
    #[sqlx(default)]
    pub author_id: Option<Vec<u8>>,
    pub title: String,
    pub description: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,

    pub state: TaskState,
    #[sqlx(default)]
    pub priority: Option<TaskPriority>,
    #[sqlx(default)]
    pub due_date: Option<chrono::NaiveDateTime>,
}
#[derive(Debug, Clone, FromRow)]
pub struct TodoReq {
    pub id: Vec<u8>,
    pub author_id: Vec<u8>,
    pub title: String,
    pub description: String,

    pub state: TaskState,
    #[sqlx(default)]
    pub priority: Option<TaskPriority>,
    #[sqlx(default)]
    pub due_date: Option<chrono::NaiveDateTime>,
}
