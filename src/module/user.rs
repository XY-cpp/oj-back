use rbatis::{crud, rbdc::datetime::DateTime};
use salvo::{handler, Request, Response, Router};
use serde::{Deserialize, Serialize};

use crate::api::front::Res;
use crate::utils::db::db;

/// 对外路由接口
pub fn init_router() -> Router {
  Router::with_path("user").post(register)
}

/// 用户结构体
#[derive(Clone, Debug, Serialize, Deserialize)]
struct User {
  id: Option<i32>,
  avatar: Option<String>,
  account: Option<String>,
  password: Option<String>,
  join_time: Option<DateTime>,
  authority: Option<i32>,
}
crud!(User {});

/// 注册用户
///
/// # 前端请求格式
/// ```json
/// {
///   accout: ...
///   password: ...
/// }
/// ```
///
/// # 后端响应格式
/// `success` 或 `error`
///
#[handler]
async fn register(req: &mut Request, res: &mut Response) {
  tracing::info!("Received a register_user request.",);
  match req.parse_json::<User>().await {
    Ok(user) => {
      let query = User::select_by_column(&db.clone(), "account", &user.account).await;
      match query {
        Ok(query) => {
          if query.len() > 0 {
            res.render(Res::error("duplicate account"));
          } else {
            let mut user = user;
            user.avatar = Some(String::from("http://127.0.0.1:8001/null"));
            user.join_time = Some(DateTime::now());
            user.authority = Some(3);
            let dbinfo = User::insert(&db.clone(), &user).await;
            match dbinfo {
              Ok(dbinfo) => {
                tracing::info!("{}", dbinfo);
                res.render(Res::success());
              }
              Err(e) => {
                tracing::error!("{:?}", e);
                res.render(Res::error("database insertion failed"));
              }
            }
          }
        }
        Err(e) => {
          tracing::error!("{:?}", e);
          res.render(Res::error("database query failed"));
        }
      }
    }
    Err(e) => {
      tracing::error!("{:?}", e);
      res.render(Res::error("json pharse failed"));
    }
  }
}
