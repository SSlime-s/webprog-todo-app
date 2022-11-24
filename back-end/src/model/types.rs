use std::str::FromStr;

use serde::{Deserialize, Serialize};
use sqlx::{error::BoxDynError, mysql::MySqlValueRef, FromRow, MySql, Type};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VecWithTotal<T: std::fmt::Debug + Clone> {
    pub total: usize,
    pub items: Vec<T>,
}

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
    #[sqlx(default)]
    pub username: Option<String>,
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TaskState {
    Icebox,
    Todo,
    InProgress,
    Done,
}
impl FromStr for TaskState {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "icebox" => Ok(TaskState::Icebox),
            "todo" => Ok(TaskState::Todo),
            "in-progress" => Ok(TaskState::InProgress),
            "done" => Ok(TaskState::Done),
            _ => Err(()),
        }
    }
}
impl ToString for TaskState {
    fn to_string(&self) -> String {
        match self {
            TaskState::Icebox => "icebox".to_string(),
            TaskState::Todo => "todo".to_string(),
            TaskState::InProgress => "in-progress".to_string(),
            TaskState::Done => "done".to_string(),
        }
    }
}
impl sqlx::Decode<'_, MySql> for TaskState {
    fn decode(value: MySqlValueRef<'_>) -> Result<Self, BoxDynError> {
        let s = <&str as sqlx::Decode<MySql>>::decode(value)?;
        TaskState::from_str(s).map_err(|_| "invalid TaskState".into())
    }
}
impl sqlx::Encode<'_, MySql> for TaskState {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> sqlx::encode::IsNull {
        self.to_string().encode_by_ref(buf)
    }
}
impl Type<MySql> for TaskState {
    fn type_info() -> <MySql as sqlx::Database>::TypeInfo {
        <str as Type<MySql>>::type_info()
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TaskPriority {
    Low,
    Medium,
    High,
}
impl FromStr for TaskPriority {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "low" => Ok(TaskPriority::Low),
            "medium" => Ok(TaskPriority::Medium),
            "high" => Ok(TaskPriority::High),
            _ => Err(()),
        }
    }
}
impl ToString for TaskPriority {
    fn to_string(&self) -> String {
        match self {
            TaskPriority::Low => "low".to_string(),
            TaskPriority::Medium => "medium".to_string(),
            TaskPriority::High => "high".to_string(),
        }
    }
}
impl sqlx::Decode<'_, MySql> for TaskPriority {
    fn decode(value: MySqlValueRef<'_>) -> Result<Self, BoxDynError> {
        let s = <&str as sqlx::Decode<MySql>>::decode(value)?;
        TaskPriority::from_str(s).map_err(|_| "invalid TaskPriority".into())
    }
}
impl sqlx::Encode<'_, MySql> for TaskPriority {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> sqlx::encode::IsNull {
        self.to_string().encode_by_ref(buf)
    }
}
impl Type<MySql> for TaskPriority {
    fn type_info() -> <MySql as sqlx::Database>::TypeInfo {
        <str as Type<MySql>>::type_info()
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct Todo {
    pub id: Vec<u8>,
    pub author_id: Option<Vec<u8>>,
    pub title: String,
    pub description: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,

    pub state: TaskState,
    pub priority: Option<TaskPriority>,
    pub due_date: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, Clone, FromRow)]
pub struct TodoReq {
    pub id: Vec<u8>,
    pub author_id: Option<Vec<u8>>,
    pub title: String,
    pub description: String,

    pub state: TaskState,
    pub priority: Option<TaskPriority>,
    pub due_date: Option<chrono::NaiveDateTime>,
}
