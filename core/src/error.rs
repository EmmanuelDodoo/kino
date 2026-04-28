pub use anyhow::{Context, Error, Result, anyhow, bail};
use std::fmt::Display;

pub trait ContextLog<T, E>: Context<T, E> {
    fn ctx_log<C>(self, context: C) -> Option<T>
    where
        C: Display + Send + Sync + 'static;

    fn with_ctx_log<C, F>(self, f: F) -> Option<T>
    where
        C: Display + Send + Sync + 'static,
        F: FnOnce() -> C;
}

pub trait Log {
    type Output;

    fn log_err(self) -> Self::Output;

    fn log_ctx<C>(self, context: C) -> Self::Output
    where
        C: Display + Send + Sync + 'static;

    fn with_ctx_log<C, F>(self, f: F) -> Self::Output
    where
        C: Display + Send + Sync + 'static,
        F: FnOnce() -> C;
}

impl<T> Log for std::result::Result<T, anyhow::Error> {
    type Output = Option<T>;

    fn log_err(self) -> Self::Output {
        match self {
            Ok(x) => Some(x),
            Err(error) => {
                error.log_err();
                None
            }
        }
    }

    fn log_ctx<C>(self, context: C) -> Self::Output
    where
        C: Display + Send + Sync + 'static,
    {
        self.context(context).log_err()
    }

    fn with_ctx_log<C, F>(self, f: F) -> Self::Output
    where
        C: Display + Send + Sync + 'static,
        F: FnOnce() -> C,
    {
        self.with_context(f).log_err()
    }
}

impl Log for anyhow::Error {
    type Output = Self;

    fn log_err(self) -> Self::Output {
        tracing::error!("{self:?}\n");
        self
    }

    fn log_ctx<C>(self, context: C) -> Self::Output
    where
        C: Display + Send + Sync + 'static,
    {
        let new = self.context(context);
        new.log_err()
    }

    fn with_ctx_log<C, F>(self, f: F) -> Self::Output
    where
        C: Display + Send + Sync + 'static,
        F: FnOnce() -> C,
    {
        let context = f();
        self.log_ctx(context)
    }
}

impl<T, E> ContextLog<T, E> for std::result::Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn ctx_log<C>(self, context: C) -> Option<T>
    where
        C: Display + Send + Sync + 'static,
    {
        self.context(context).log_err()
    }

    fn with_ctx_log<C, F>(self, f: F) -> Option<T>
    where
        C: Display + Send + Sync + 'static,
        F: FnOnce() -> C,
    {
        self.with_context(f).log_err()
    }
}
