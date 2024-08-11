use rbatis::{crud, rbdc::datetime::DateTime};
use salvo::{handler, Request, Response, Router};
use serde::{Deserialize, Serialize};

use crate::api::front::{to_json, Res};
use crate::utils::db::db;

/// 对外路由接口
pub fn init_router() -> Router {
  Router::with_path("user")
    .push(Router::with_path("register").post(register))
    .push(Router::with_path("login").post(login))
    .push(Router::with_path("update").post(update))
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

/// 用户注册
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
  tracing::info!("Received a request to register a user.",);
  match req.parse_json::<User>().await {
    Ok(user) => {
      let query = User::select_by_column(&db.clone(), "account", &user.account).await;
      match query {
        Ok(query) => {
          if query.len() > 0 {
            tracing::info!("Duplicate account found.");
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

/// 用户登录
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
/// `success` 或 `error`, 成功响应的`data`中包含`User`结构体的所有信息
///
#[handler]
async fn login(req: &mut Request, res: &mut Response) {
  tracing::info!("Received a request to login.",);
  match req.parse_json::<User>().await {
    Ok(user) => {
      let query = User::select_by_column(&db.clone(), "account", &user.account).await;
      match query {
        Ok(query) => {
          if query.len() == 0 {
            tracing::info!("Account not found.");
            res.render(Res::error("account not found"));
          } else {
            if user.password != query[0].password {
              tracing::info!("Wrong password.");
              res.render(Res::error("wrong password"));
            } else {
              tracing::info!("User {} login successfully.", &query[0].id.unwrap());
              res.render(Res::success_data(to_json(&query[0])));
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

/// 用户修改
///
/// # 前端请求格式
/// ```json
/// {
///   id: ... //要修改的用户id
///   ......  //要修改的用户数据
/// }
/// ```
///
/// # 后端响应格式
/// `success` 或 `error`
///
#[handler]
async fn update(req: &mut Request, res: &mut Response) {
  tracing::info!("Received a request to login.",);
  match req.parse_json::<User>().await {
    Ok(user) => {
      let query = User::select_by_column(&db.clone(), "id", &user.id).await;
      match query {
        Ok(query) => {
          if query.len() == 0 {
            tracing::info!("User not found.");
            res.render(Res::error("user not found"));
          } else {
            let mut new_user = query[0].clone();
            if let Some(avatar) = user.avatar {
              new_user.avatar = Some(avatar);
            }
            if let Some(account) = user.account {
              new_user.account = Some(account);
            }
            if let Some(password) = user.password {
              new_user.password = Some(password);
            }
            if let Some(join_time) = user.join_time {
              new_user.join_time = Some(join_time);
            }
            if let Some(authority) = user.authority {
              new_user.authority = Some(authority);
            }
            let dbinfo = User::update_by_column(&db.clone(), &new_user, "id").await;
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
