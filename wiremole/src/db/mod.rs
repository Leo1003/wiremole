use std::iter::IntoIterator;

pub mod models;
pub mod schema;

// Idea from https://github.com/dani-garcia/vaultwarden/blob/main/src/db/mod.rs
pub trait IntoModel {
    type Output;
    type Error;

    fn into_model(self) -> Result<Self::Output, Self::Error>;
}

impl<I: IntoIterator<Item = T>, T: IntoModel> IntoModel for I {
    type Output = Vec<T::Output>;
    type Error = T::Error;

    #[inline(always)]
    fn into_model(self) -> Result<Self::Output, Self::Error> {
        self.into_iter().map(IntoModel::into_model).collect()
    }
}

pub trait FromModel<M>: Sized {
    type Error;

    fn from_model(model: M) -> Result<Self, Self::Error>;
}
