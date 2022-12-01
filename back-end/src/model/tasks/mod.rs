use std::{fmt::Display, str::FromStr};

use sqlx::{mysql::MySqlArguments, Acquire, MySql};

use super::{types::VecWithTotal, Update};
use crate::utils::ulid_to_binary;

use super::types;

#[derive(Debug, Clone, Copy)]
pub enum Order {
    Asc,
    Desc,
}
impl Order {
    pub fn to_query(self) -> String {
        match self {
            Order::Asc => "ASC".to_string(),
            Order::Desc => "DESC".to_string(),
        }
    }
}
impl FromStr for Order {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "asc" => Ok(Order::Asc),
            "desc" => Ok(Order::Desc),
            _ => Err(anyhow::anyhow!("Invalid order")),
        }
    }
}

#[derive(Debug, Clone)]
pub enum SortedBy {
    CreatedAt(Order),
    UpdatedAt(Order),
    Priority(Order),
    PriorityAndCreatedAt(Order, Order),
    PriorityAndUpdatedAt(Order, Order),
}
impl SortedBy {
    const PRIORITY_CASE_QUERY: &'static str = r#"CASE
            WHEN (`priority` = 'low') THEN 0
            WHEN (`priority` IS NULL) THEN 1
            WHEN (`priority` = 'medium') THEN 2
            WHEN (`priority` = 'high') THEN 3
            ELSE 4 END"#;

    pub fn to_query(&self) -> String {
        let mut query = Vec::new();
        query.push("ORDER BY".to_string());

        match self {
            SortedBy::CreatedAt(order) => {
                query.push(format!("`created_at` {}", order.to_query()));
            }
            SortedBy::UpdatedAt(order) => {
                query.push(format!("`updated_at` {}", order.to_query()));
            }
            SortedBy::Priority(order)
            | SortedBy::PriorityAndCreatedAt(order, _)
            | SortedBy::PriorityAndUpdatedAt(order, _) => {
                query.push(format!(
                    "{} {}",
                    Self::PRIORITY_CASE_QUERY,
                    order.to_query()
                ));
                match self {
                    SortedBy::PriorityAndCreatedAt(_, order) => {
                        query.push(format!(", `created_at` {}", order.to_query()));
                    }
                    SortedBy::PriorityAndUpdatedAt(_, order) => {
                        query.push(format!(", `updated_at` {}", order.to_query()));
                    }
                    SortedBy::Priority(_) => {}
                    _ => unreachable!(),
                }
            }
        }

        query.join(" ")
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
    phrase: Option<String>,
    limit: Option<Limit>,
    sorted_by: Option<SortedBy>,

    state_filter: Option<Vec<types::TaskState>>,
) -> anyhow::Result<VecWithTotal<types::Todo>> {
    let mut conn = conn.acquire().await?;

    let mut query = "SELECT SQL_CALC_FOUND_ROWS * FROM `todos` WHERE `author_id` = ?".to_string();
    if phrase.is_some() {
        query.push_str(" AND (`title` LIKE ? OR `description` LIKE ?)");
    }
    if let Some(state_filter) = state_filter {
        query.push_str(" AND `state` IN (");
        query.push_str(
            &state_filter
                .iter()
                .map(|s| format!("'{}'", s.to_string()))
                .collect::<Vec<_>>()
                .join(", "),
        );
        query.push(')');
    }
    query.push_str(&format!(
        " {}",
        SortedBy::CreatedAt(Order::Desc).to_query().as_str()
    ));
    query.push_str(&format!(
        " {}",
        limit.map(|l| l.to_prepared_query()).unwrap_or_default()
    ));
    query.push(';');

    let bin_id = ulid_to_binary(author_id);

    let building_query = {
        let mut building_query =
            sqlx::query_as::<_, types::Todo>(query.as_str()).bind(bin_id.as_slice());

        if let Some(phrase) = phrase {
            let phrase = format!("%{}%", phrase);
            building_query = building_query.bind(phrase.clone()).bind(phrase);
        }

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

    let total = sqlx::query_as::<_, (i64,)>("SELECT FOUND_ROWS()")
        .fetch_one(&mut *conn)
        .await?
        .0 as usize;

    Ok(VecWithTotal { total, items: rows })
}

pub async fn get_task(
    conn: impl Acquire<'_, Database = MySql>,
    task_id: ulid::Ulid,
) -> anyhow::Result<Option<types::Todo>> {
    let mut conn = conn.acquire().await?;

    let query = "SELECT * FROM `todos` WHERE `id` = ?;";

    let bin_task_id = ulid_to_binary(task_id);

    let row = sqlx::query_as::<_, types::Todo>(query)
        .bind(bin_task_id.as_slice())
        .fetch_optional(&mut *conn)
        .await?;

    Ok(row)
}

pub async fn get_task_with_lock(
    conn: impl Acquire<'_, Database = MySql>,
    task_id: ulid::Ulid,
) -> anyhow::Result<Option<types::Todo>> {
    let mut conn = conn.acquire().await?;

    let query = "SELECT * FROM `todos` WHERE `id` = ? FOR UPDATE;";

    let bin_task_id = ulid_to_binary(task_id);

    let row = sqlx::query_as::<_, types::Todo>(query)
        .bind(bin_task_id.as_slice())
        .fetch_optional(&mut *conn)
        .await?;

    Ok(row)
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

    let building_query = update
        .bind_query(sqlx::query(query.as_str()))
        .bind(bin_id.as_slice());

    building_query.execute(&mut *conn).await?;

    Ok(())
}

pub async fn delete_task(
    conn: impl Acquire<'_, Database = MySql>,
    id: ulid::Ulid,
) -> anyhow::Result<()> {
    let mut conn = conn.acquire().await?;

    let query = "DELETE FROM `todos` WHERE `id` = ?;";

    let bin_id = ulid_to_binary(id);

    sqlx::query(query)
        .bind(bin_id.as_slice())
        .execute(&mut *conn)
        .await?;

    Ok(())
}
