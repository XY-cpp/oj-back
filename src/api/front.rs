use salvo::writing::Json;
use serde::Serialize;

pub fn to_string<T: Serialize>(data: &T) -> String {
  serde_json::to_string(data).unwrap()
}

#[derive(Serialize, Default)]
pub struct Res {
  pub status: Option<String>,
  pub message: Option<String>,
  pub data: Option<String>,
}

impl Res {
  pub fn new() -> Self {
    Res::default()
  }
  pub fn success() -> Json<Self> {
    Res::new().status("success").to_json()
  }
  pub fn error<T: ToString>(message: T) -> Json<Self> {
    Res::new().status("error").message(message).to_json()
  }
  pub fn status<T: ToString>(mut self, status: T) -> Self {
    self.status = Some(status.to_string());
    self
  }
  pub fn message<T: ToString>(mut self, message: T) -> Self {
    self.message = Some(message.to_string());
    self
  }
  pub fn data<T: ToString>(mut self, data: T) -> Self {
    self.data = Some(data.to_string());
    self
  }
  pub fn to_json(self) -> Json<Self> {
    Json(self)
  }
}
