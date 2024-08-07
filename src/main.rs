// #[macro_use]
// extern crate rbatis;

use salvo::prelude::*;

mod utils;
use utils::{config::config, db::init_mysql};

#[handler]
async fn hello() -> String {
  return String::from("Hello world");
}

#[tokio::main]
async fn main() {
  tracing_subscriber::fmt().init();

  init_mysql(
    &config.mysql.username,
    &config.mysql.password,
    &config.mysql.host,
    &config.mysql.port,
    &config.mysql.database,
  )
  .await;

  let router = Router::new().get(hello);
  let listen_addr = format!("{}:{}", config.server.host, config.server.port);
  let acceptor = TcpListener::new(listen_addr).bind().await;
  let server = Server::new(acceptor);

  let handle = server.handle();
  tokio::spawn(async move {
    tokio::signal::ctrl_c().await.unwrap();
    handle.stop_graceful(std::time::Duration::from_secs(5));
  });
  server.serve(router).await;
}
