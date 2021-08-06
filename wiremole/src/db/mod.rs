use std::iter::IntoIterator;

pub mod models;
pub mod schema;

// Idea from https://github.com/dani-garcia/vaultwarden/blob/main/src/db/mod.rs
pub trait FromDb {
    type Output;
    type Error;

    #[allow(clippy::wrong_self_convention)]
    fn from_db(self) -> Result<Self::Output, Self::Error>;
}

impl<I: IntoIterator<Item = T>, T: FromDb> FromDb for I {
    type Output = Vec<T::Output>;
    type Error = T::Error;

    #[inline(always)]
    fn from_db(self) -> Result<Self::Output, Self::Error> {
        self.into_iter().map(FromDb::from_db).collect()
    }
}
