use salvo::writing::Json;
use serde::Serialize;
use serde_json::Value;

#[derive(Serialize, Default)]
pub struct Res {
  pub status: Option<String>,
  pub message: Option<String>,
  pub data: Option<Value>,
}

impl Res {
  pub fn new() -> Self {
    Res::default()
  }
  pub fn success() -> Json<Self> {
    Res::new().status("success").to_json()
  }
  pub fn success_data(data: Value) -> Json<Self> {
    Res::new().status("success").data(data).to_json()
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
  pub fn data(mut self, data: Value) -> Self {
    self.data = Some(data);
    self
  }
  pub fn to_json(self) -> Json<Self> {
    Json(self)
  }
}
