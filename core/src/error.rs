use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
  #[error("{0}")]
  Msg(String),
  #[error("io: {0}")]
  Io(#[from] std::io::Error),
  #[error("json: {0}")]
  Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, CoreError>;

impl CoreError {
  pub fn msg(s: impl Into<String>) -> Self {
    Self::Msg(s.into())
  }
}

impl From<CoreError> for String {
  fn from(value: CoreError) -> Self {
    value.to_string()
  }
}
