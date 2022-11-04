use std::{fmt::Display, str::FromStr};

use sqlx::{Acquire, MySql};

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
