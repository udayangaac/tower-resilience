use thiserror::Error;

#[derive(Error, Debug)]
pub enum ResilienceError<E> {
    #[error("request timed out")]
    Timeout,

    #[error("inner service error: {0}")]
    Inner(E),
}
