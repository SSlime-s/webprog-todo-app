use sqlx::mysql::MySqlArguments;

pub mod tasks;
pub mod types;
pub mod users;

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
impl<T> Update<Option<T>> {
    pub fn transpose(self) -> Option<Update<T>> {
        match self {
            Self::Set(Some(t)) => Some(Update::Set(t)),
            Self::Set(None) => None,
            Self::Nop => Some(Update::Nop),
        }
    }
}
impl<T, E> Update<Result<T, E>> {
    pub fn transpose(self) -> Result<Update<T>, E> {
        match self {
            Self::Set(Ok(t)) => Ok(Update::Set(t)),
            Self::Set(Err(e)) => Err(e),
            Self::Nop => Ok(Update::Nop),
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
impl<'de, T> serde::Deserialize<'de> for Update<T>
where
    T: serde::Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let t = T::deserialize(deserializer)?;
        Ok(Self::Set(t))
    }
}
