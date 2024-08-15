#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error("{0}")]
  JsonParseError(#[from] salvo::http::ParseError),

  #[error("{0}")]
  RbatisError(#[from] rbatis::rbdc::Error),

  #[error("{0}")]
  TokenError(#[from] jsonwebtoken::errors::Error),

  #[error("DuplicateData: {0}")]
  DuplicateData(String),

  #[error("NoToken: {0}")]
  NoToken(String),

  #[error("EmptyData: {0}")]
  EmptyData(String),

  #[error("WrongPassword: {0}")]
  WrongPassword(String),

  #[error("DataNotFound: {0}")]
  DataNotFound(String),

  #[error("NoAuthority: {0}")]
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
          let text = e.to_string();
          let index = text.find(':').unwrap();
          $response.render(
            Res::new()
              .status("error")
              .message(text[..index].to_string())
              .data(json!(text[index + 2..]))
              .to_json(),
          );
        }
      }
    }
  };
}
