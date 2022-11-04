use std::{fmt::Display, str::FromStr};

use sqlx::{mysql::MySqlArguments, Acquire, MySql};

use crate::utils::ulid_to_binary;

use super::types;

#[derive(Debug, Clone)]
pub enum SortedBy {
    CreatedAt,
    UpdatedAt,
}
impl FromStr for SortedBy {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self> {
        match s {
            "created_at" => Ok(Self::CreatedAt),
            "updated_at" => Ok(Self::UpdatedAt),
            _ => Err(anyhow::anyhow!("Invalid sort_by")),
        }
    }
}
impl Display for SortedBy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CreatedAt => write!(f, "created_at"),
            Self::UpdatedAt => write!(f, "updated_at"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Limit {
    LimitOffset(usize, usize),
    Limit(usize),
}
impl Limit {
    pub fn to_query(self) -> String {
        match self {
            Self::LimitOffset(limit, offset) => format!("LIMIT {} OFFSET {}", limit, offset),
            Self::Limit(limit) => format!("LIMIT {}", limit),
        }
    }

    pub fn to_prepared_query(self) -> String {
        match self {
            Self::LimitOffset(_, _) => "LIMIT ? OFFSET ?".to_string(),
            Self::Limit(_) => "LIMIT ?".to_string(),
        }
    }
}

pub async fn get_tasks(
    conn: impl Acquire<'_, Database = MySql>,
    author_id: ulid::Ulid,
    limit: Option<Limit>,
    sorted_by: Option<SortedBy>,
) -> anyhow::Result<Vec<types::Todo>> {
    let mut conn = conn.acquire().await?;

    let query = format!(
        "SELECT * FROM `todos` WHERE `author_id` = ? {};",
        limit.map(|l| l.to_prepared_query()).unwrap_or_default()
    );

    let bin_id = ulid_to_binary(author_id);

    let building_query = {
        let mut building_query =
            sqlx::query_as::<_, types::Todo>(query.as_str()).bind(bin_id.as_slice());

        match limit {
            Some(Limit::LimitOffset(limit, offset)) => {
                building_query = building_query.bind(limit as i64).bind(offset as i64)
            }
            Some(Limit::Limit(limit)) => building_query = building_query.bind(limit as i64),
            None => (),
        }
        building_query
    };

    let rows = building_query.fetch_all(&mut *conn).await?;

    Ok(rows)
}

pub async fn insert_task(
    conn: impl Acquire<'_, Database = MySql>,
    task: types::TodoReq,
) -> anyhow::Result<()> {
    let mut conn = conn.acquire().await?;

    let query = r#"
        INSERT INTO `todos`
            (`id`, `author_id`, `title`, `description`, `state`, `priority`, `due_date`)
            VALUES (?, ?, ?, ?, ?, ?, ?);"#;

    let priority_str: Option<String> = task.priority.map(|p| p.to_string());

    sqlx::query(query)
        .bind(task.id)
        .bind(task.author_id)
        .bind(task.title)
        .bind(task.description)
        .bind(task.state)
        .bind(priority_str)
        .bind(task.due_date)
        .execute(&mut *conn)
        .await?;

    Ok(())
}

#[derive(Debug, Clone)]
pub enum Update<T> {
    Set(T),
    Nop,
}
impl<T> Update<T> {
    pub fn unwrap(self) -> T {
        match self {
            Self::Set(t) => t,
            Self::Nop => panic!("Update::Nop"),
        }
    }

    pub fn is_nop(&self) -> bool {
        matches!(self, Self::Nop)
    }

    pub fn is_set(&self) -> bool {
        matches!(self, Self::Set(_))
    }

    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Update<U> {
        match self {
            Self::Set(t) => Update::Set(f(t)),
            Self::Nop => Update::Nop,
        }
    }

    pub fn to_prepared_query(&self, column_name: &str) -> Option<String> {
        match self {
            Self::Set(_) => Some(format!("`{}` = ?", column_name)),
            Self::Nop => None,
        }
    }
}
impl<'a, T> Update<T>
where
    &'a T: 'a + Send + sqlx::Encode<'a, sqlx::MySql> + sqlx::Type<sqlx::MySql>,
{
    pub fn bind_query(
        &'a self,
        query: sqlx::query::Query<'a, sqlx::MySql, MySqlArguments>,
    ) -> sqlx::query::Query<'a, sqlx::MySql, MySqlArguments> {
        match self {
            Self::Set(t) => query.bind(t),
            Self::Nop => query,
        }
    }
}
impl<T> Default for Update<T> {
    fn default() -> Self {
        Self::Nop
    }
}

#[derive(Debug, Clone, Default)]
pub struct UpdateTask {
    pub title: Update<String>,
    pub description: Update<String>,
    pub state: Update<types::TaskState>,
    pub priority: Update<Option<types::TaskPriority>>,
    pub due_date: Update<Option<chrono::NaiveDateTime>>,
}
impl UpdateTask {
    fn to_prepared_query(&self) -> String {
        let mut query = Vec::new();

        if let Some(q) = self.title.to_prepared_query("title") {
            query.push(q);
        }
        if let Some(q) = self.description.to_prepared_query("description") {
            query.push(q);
        }
        if let Some(q) = self.state.to_prepared_query("state") {
            query.push(q);
        }
        if let Some(q) = self.priority.to_prepared_query("priority") {
            query.push(q);
        }
        if let Some(q) = self.due_date.to_prepared_query("due_date") {
            query.push(q);
        }

        query.join(", ")
    }

    pub fn bind_query<'a>(
        &'a self,
        query: sqlx::query::Query<'a, sqlx::MySql, MySqlArguments>,
    ) -> sqlx::query::Query<'a, sqlx::MySql, MySqlArguments> {
        let mut query = self.title.bind_query(query);
        query = self.description.bind_query(query);
        query = self.state.bind_query(query);
        query = self.priority.bind_query(query);
        query = self.due_date.bind_query(query);

        query
    }

    pub fn is_nop(&self) -> bool {
        self.title.is_nop()
            && self.description.is_nop()
            && self.state.is_nop()
            && self.priority.is_nop()
            && self.due_date.is_nop()
    }
}

pub async fn update_task(
    conn: impl Acquire<'_, Database = MySql>,
    id: ulid::Ulid,
    update: UpdateTask,
) -> anyhow::Result<()> {
    let mut conn = conn.acquire().await?;

    if update.is_nop() {
        return Ok(());
    }

    let query = format!(
        "UPDATE `todos` SET {} WHERE `id` = ?;",
        update.to_prepared_query()
    );

    let bin_id = ulid_to_binary(id);

    let building_query = update.bind_query(sqlx::query(query.as_str()).bind(bin_id.as_slice()));

    building_query.execute(&mut *conn).await?;

    Ok(())
}
