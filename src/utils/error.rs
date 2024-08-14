#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error("Json parse error: {0}")]
  JsonParseError(#[from] salvo::http::ParseError),

  #[error("Rbatis error: {0}")]
  RbatisError(#[from] rbatis::rbdc::Error),

  #[error("Token error: {0}")]
  TokenError(#[from] jsonwebtoken::errors::Error),

  #[error("Duplicate data: {0}")]
  DuplicateData(String),

  #[error("Wrong data format: {0}")]
  WrongDataFormat(String),

  #[error("Wrong password: {0}")]
  WrongPassword(String),

  #[error("Data not found: {0}")]
  DataNotFound(String),

  #[error("No authority: {0}")]
  NoAuthority(String),
}

/// 错误生成宏
#[macro_export]
macro_rules! generate_error {
  ($variant:path, $dict:expr) => {
    Err($variant($dict))
  };
}

/// 错误处理宏
#[macro_export]
macro_rules! handle_error {
  ($operation:expr, $response:expr) => {
    if let Err(e) = $operation.await {
      match e {
        self::Error::JsonParseError(_)
        | self::Error::RbatisError(_)
        | self::Error::TokenError(_) => {
          tracing::error!("{}", e);
          $response.render(
            Res::new()
              .status("error")
              .message("internal error")
              .data(json!(e.to_string()))
              .to_json(),
          );
        }
        _ => {
          tracing::warn!("{}", e);
          $response.render(
            crate::api::front::Res::new()
              .status("error")
              .message("data error")
              .data(json!(e.to_string()))
              .to_json(),
          );
        }
      }
    }
  };
}
