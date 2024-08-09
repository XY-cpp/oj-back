mod api;
mod module;
mod utils;

use module::router::init_router;
use salvo::{conn::TcpListener, Listener, Server};
use utils::{config::config, db::init_mysql};

#[tokio::main]
async fn main() {
  tracing_subscriber::fmt()
    .event_format(
      tracing_subscriber::fmt::format()
        .with_file(true)
        .with_line_number(true),
    )
    .init();
  init_mysql(
    &config.mysql.username,
    &config.mysql.password,
    &config.mysql.host,
    &config.mysql.port,
    &config.mysql.database,
  )
  .await;

  let router = init_router();
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
