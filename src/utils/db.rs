use rbatis::RBatis;
use rbdc_mysql::driver::MysqlDriver;

lazy_static::lazy_static! {
  pub static ref db: RBatis = RBatis::new();
}

pub async fn init_mysql(
  username: &String,
  password: &String,
  host: &String,
  port: &u16,
  database: &String,
) {
  let mysql_url = format!(
    "mysql://{}:{}@{}:{}/{}",
    username, password, host, port, database
  );
  db.init(MysqlDriver {}, &mysql_url).unwrap();
}
