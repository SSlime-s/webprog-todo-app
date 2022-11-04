use ulid::Ulid;

pub const ULID_BIN_LEN: usize = 16;

pub fn binary_to_ulid(binary: &[u8]) -> anyhow::Result<Ulid> {
    if binary.len() != ULID_BIN_LEN {
        anyhow::bail!("Invalid binary length");
    }

    let mut bytes = [0u8; ULID_BIN_LEN];
    bytes.copy_from_slice(binary);
    let val = u128::from_be_bytes(bytes);
    Ok(val.into())
}

pub fn ulid_to_binary(ulid: Ulid) -> [u8; ULID_BIN_LEN] {
    u128::from(ulid).to_be_bytes()
}

pub async fn check_is_logged_in(
    session: actix_session::Session,
    conn: impl sqlx::Acquire<'_, Database = sqlx::MySql>,
) -> anyhow::Result<ulid::Ulid> {
    use crate::model::users::is_valid_id;

    let user_id = session.get::<String>("user_id")?;
    if user_id.is_none() {
        anyhow::bail!("Unauthorized");
    }
    let user_id = user_id.unwrap();
    let user_ulid = ulid::Ulid::from_string(&user_id)?;
    let is_valid = is_valid_id(conn, user_ulid).await?;
    if !is_valid {
        anyhow::bail!("Invalid user id");
    }

    Ok(user_ulid)
}
