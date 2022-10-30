use sqlx::{Acquire, MySql, MySqlPool, Row};

use crate::utils::ulid_to_binary;

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

pub async fn get_user_from_username(
    pool: &MySqlPool,
    username: &str,
) -> anyhow::Result<Option<crate::model::types::User>> {
    let query = "SELECT * FROM `users` WHERE `username` = ?;";
    let row = sqlx::query_as::<_, crate::model::types::User>(query)
        .bind(username)
        .fetch_optional(pool)
        .await?;
    Ok(row)
}
