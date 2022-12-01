use sqlx::{mysql::MySqlArguments, Acquire, MySql, Row};

use crate::utils::ulid_to_binary;

use super::Update;

pub async fn is_username_exists(
    conn: impl Acquire<'_, Database = MySql>,
    username: &str,
) -> anyhow::Result<bool> {
    let mut conn = conn.acquire().await?;

    let query = "SELECT COUNT(*) FROM `users` WHERE `username` = ?;";
    let count = sqlx::query(query)
        .bind(username)
        .fetch_one(&mut *conn)
        .await?
        .get::<i32, _>(0);
    Ok(count > 0)
}

pub async fn insert_user(
    conn: impl Acquire<'_, Database = MySql>,
    id: Option<ulid::Ulid>,
    username: &str,
    display_name: &str,
    hashed_password: &[u8],
) -> anyhow::Result<()> {
    let mut conn = conn.acquire().await?;

    let query = r#"
        INSERT INTO `users`
            (`id`, `username`, `display_name`, `hashed_password`)
            VALUES (?, ?, ?, ?);"#;
    let id = id.unwrap_or_else(ulid::Ulid::new);
    let bin_id = ulid_to_binary(id);

    sqlx::query(query)
        .bind(bin_id.as_slice())
        .bind(username)
        .bind(display_name)
        .bind(hashed_password)
        .execute(&mut *conn)
        .await?;

    Ok(())
}

pub async fn remove_user(
    conn: impl Acquire<'_, Database = MySql>,
    id: ulid::Ulid,
) -> anyhow::Result<()> {
    let mut conn = conn.acquire().await?;

    let query = "UPDATE `users` SET `deleted_at` = NOW(), `username` = NULL WHERE `id` = ?;";
    let bin_id = ulid_to_binary(id);

    sqlx::query(query)
        .bind(bin_id.as_slice())
        .execute(&mut *conn)
        .await?;

    Ok(())
}

#[derive(Debug, Clone, Default)]
pub struct UpdateUser {
    pub username: Update<String>,
    pub display_name: Update<String>,
    pub hashed_password: Update<Vec<u8>>,
}
impl UpdateUser {
    pub fn to_prepared_query(&self) -> String {
        let mut query = Vec::new();

        if let Some(q) = self.username.to_prepared_query("username") {
            query.push(q);
        }
        if let Some(q) = self.display_name.to_prepared_query("display_name") {
            query.push(q);
        }
        if let Some(q) = self.hashed_password.to_prepared_query("hashed_password") {
            query.push(q);
        }

        query.join(", ")
    }

    pub fn bind_query<'a>(
        &'a self,
        query: sqlx::query::Query<'a, sqlx::MySql, MySqlArguments>,
    ) -> sqlx::query::Query<'a, sqlx::MySql, MySqlArguments> {
        let mut query = self.username.bind_query(query);
        query = self.display_name.bind_query(query);
        query = self.hashed_password.bind_query(query);
        query
    }

    pub fn is_nop(&self) -> bool {
        self.display_name.is_nop() && self.hashed_password.is_nop()
    }
}

pub async fn verify_password(
    conn: impl Acquire<'_, Database = MySql>,
    id: ulid::Ulid,
    hashed_password: &[u8],
) -> anyhow::Result<bool> {
    let mut conn = conn.acquire().await?;

    let query = "SELECT COUNT(*) FROM `users` WHERE `id` = ? AND `hashed_password` = ?;";
    let bin_id = ulid_to_binary(id);

    let count = sqlx::query(query)
        .bind(bin_id.as_slice())
        .bind(hashed_password)
        .fetch_one(&mut *conn)
        .await?
        .get::<i32, _>(0);
    Ok(count > 0)
}

pub async fn update_user(
    conn: impl Acquire<'_, Database = MySql>,
    id: ulid::Ulid,
    update: UpdateUser,
) -> anyhow::Result<()> {
    let mut conn = conn.acquire().await?;

    if update.is_nop() {
        return Ok(());
    }

    let query = format!(
        "UPDATE `users` SET {} WHERE `id` = ?;",
        update.to_prepared_query()
    );
    let bin_id = ulid_to_binary(id);

    let building_query = update
        .bind_query(sqlx::query(query.as_str()))
        .bind(bin_id.as_slice());

    building_query.execute(&mut *conn).await?;

    Ok(())
}

pub async fn get_user_from_username(
    conn: impl Acquire<'_, Database = MySql>,
    username: &str,
) -> anyhow::Result<Option<crate::model::types::User>> {
    let mut conn = conn.acquire().await?;

    let query = "SELECT * FROM `users` WHERE `username` = ?;";
    let row = sqlx::query_as::<_, crate::model::types::User>(query)
        .bind(username)
        .fetch_optional(&mut *conn)
        .await?;
    Ok(row)
}

pub async fn get_user(
    conn: impl Acquire<'_, Database = MySql>,
    id: ulid::Ulid,
) -> anyhow::Result<Option<crate::model::types::User>> {
    let mut conn = conn.acquire().await?;

    let query = "SELECT * FROM `users` WHERE `id` = ? AND `deleted_at` IS NULL;";
    let bin_id = ulid_to_binary(id);
    let row = sqlx::query_as::<_, crate::model::types::User>(query)
        .bind(bin_id.as_slice())
        .fetch_optional(&mut *conn)
        .await?;
    Ok(row)
}

pub async fn is_valid_id(
    conn: impl Acquire<'_, Database = MySql>,
    id: ulid::Ulid,
) -> anyhow::Result<bool> {
    let mut conn = conn.acquire().await?;

    let query = "SELECT COUNT(*) FROM `users` WHERE `id` = ? AND `deleted_at` IS NULL;";
    let bin_id = ulid_to_binary(id);
    let count = sqlx::query(query)
        .bind(bin_id.as_slice())
        .fetch_one(&mut *conn)
        .await?
        .get::<i32, _>(0);
    Ok(count > 0)
}
