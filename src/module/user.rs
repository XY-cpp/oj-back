use rbatis::{crud, rbdc::datetime::DateTime};
use salvo::http::cookie::Cookie;
use salvo::{handler, Request, Response, Router};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::api::front::Res;
use crate::utils::authority::{check_authority, Authority, Jwt};
use crate::utils::db::db;

/// 对外路由接口
pub fn init_router() -> Router {
  Router::with_path("user")
    .push(Router::with_path("register").post(register))
    .push(Router::with_path("login").post(login))
    .push(Router::with_path("update").post(update))
    .push(Router::with_path("query").post(query))
    .push(Router::with_path("delete").post(delete))
}

/// 用户结构体
#[derive(Clone, Debug, Serialize, Deserialize)]
struct User {
  id: Option<i32>,
  avatar: Option<String>,
  account: Option<String>,
  password: Option<String>,
  join_time: Option<DateTime>,
  auth: Option<Authority>,
}
crud!(User {});

/// 用户注册
///
/// # 前端请求地址
/// `/user/register`
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
async fn register(request: &mut Request, response: &mut Response) {
  tracing::info!("Received a request to register a user.",);
  match request.parse_json::<User>().await {
    Ok(user) => {
      let dbres = User::select_by_column(&db.clone(), "account", &user.account).await;
      match dbres {
        Ok(dbres) => {
          if dbres.len() > 0 {
            tracing::info!("Duplicate account found.");
            response.render(Res::error("duplicate account"));
          } else {
            let mut user = user;
            user.avatar = Some(String::from("http://127.0.0.1:8001/null"));
            user.join_time = Some(DateTime::now());
            user.auth = Some(Authority::User);
            let dbinfo = User::insert(&db.clone(), &user).await;
            match dbinfo {
              Ok(dbinfo) => {
                tracing::info!("{}", dbinfo);
                response.render(Res::success());
              }
              Err(e) => {
                tracing::error!("{:?}", e);
                response.render(Res::error("database insertion failed"));
              }
            }
          }
        }
        Err(e) => {
          tracing::error!("{:?}", e);
          response.render(Res::error("database query failed"));
        }
      }
    }
    Err(e) => {
      tracing::error!("{:?}", e);
      response.render(Res::error("json pharse failed"));
    }
  }
}

/// 用户登录
///
/// # 前端请求地址
/// `/user/login`
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
async fn login(request: &mut Request, response: &mut Response) {
  tracing::info!("Received a request to login.",);
  match request.parse_json::<User>().await {
    Ok(user) => {
      let dbres = User::select_by_column(&db.clone(), "account", &user.account).await;
      match dbres {
        Ok(dbres) => {
          if dbres.len() == 0 {
            tracing::info!("Account not found.");
            response.render(Res::error("account not found"));
          } else {
            if user.password != dbres[0].password {
              tracing::info!("Wrong password.");
              response.render(Res::error("wrong password"));
            } else {
              let user = dbres[0].clone();
              match Jwt::encode(user.id.unwrap(), user.auth.clone().unwrap()) {
                Ok((token, exp)) => {
                  tracing::info!(
                    "User {} login successfully with token {}.",
                    user.id.unwrap(),
                    &token
                  );
                  let mut cookie = Cookie::new("token", token);
                  cookie.set_expires(exp);
                  response.add_cookie(cookie);
                  response.render(Res::success_data(json!(&user)));
                }
                Err(e) => {
                  tracing::error!("{:?}", e);
                  response.render(Res::error("token generation failed"));
                }
              }
            }
          }
        }
        Err(e) => {
          tracing::error!("{:?}", e);
          response.render(Res::error("database query failed"));
        }
      }
    }
    Err(e) => {
      tracing::error!("{:?}", e);
      response.render(Res::error("json pharse failed"));
    }
  }
}

/// 用户修改
///
/// # 前端请求地址
/// `/user/update`
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
async fn update(request: &mut Request, response: &mut Response) {
  tracing::info!("Received a request to update a user.",);
  if let None = request.cookie("token") {
    tracing::info!("User not login.");
    response.render(Res::error("user not login"));
    return;
  }
  match request.parse_json::<User>().await {
    Ok(user) => {
      let dbres = User::select_by_column(&db.clone(), "id", &user.id).await;
      match dbres {
        Ok(dbres) => {
          if dbres.len() == 0 {
            tracing::info!("User not found.");
            response.render(Res::error("user not found"));
          } else {
            match check_authority(
              request.cookie("token").unwrap().value().to_string(),
              user.id.unwrap(),
              Authority::Admin,
            ) {
              false => {
                tracing::info!("User {} has no authority.", user.id.unwrap());
                response.render(Res::error("no authority"));
              }
              true => {
                let mut new_user = dbres[0].clone();
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
                if let Some(authority) = user.auth {
                  new_user.auth = Some(authority);
                }
                let dbinfo = User::update_by_column(&db.clone(), &new_user, "id").await;
                match dbinfo {
                  Ok(dbinfo) => {
                    tracing::info!("{}", dbinfo);
                    response.render(Res::success());
                  }
                  Err(e) => {
                    tracing::error!("{:?}", e);
                    response.render(Res::error("database insertion failed"));
                  }
                }
              }
            }
          }
        }
        Err(e) => {
          tracing::error!("{:?}", e);
          response.render(Res::error("database query failed"));
        }
      }
    }
    Err(e) => {
      tracing::error!("{:?}", e);
      response.render(Res::error("json pharse failed"));
    }
  }
}

/// 用户查询
///
/// # 前端请求地址
/// `/user/query`
///
/// # 前端请求格式
/// ```json
/// {
///   id: ... //要查询的用户id
/// }
/// ```
///
/// # 后端响应格式
/// `success` 或 `error`, 成功响应的`data`中包含`User`结构体的所有信息
///
#[handler]
async fn query(request: &mut Request, response: &mut Response) {
  tracing::info!("Received a request to query.",);
  match request.parse_json::<User>().await {
    Ok(user) => {
      let dbres = User::select_by_column(&db.clone(), "id", &user.id).await;
      match dbres {
        Ok(dbres) => {
          if dbres.len() == 0 {
            tracing::info!("User not found.");
            response.render(Res::error("user not found"));
          } else {
            tracing::info!("Query successfully.");
            response.render(Res::success_data(json!(&dbres[0])));
          }
        }
        Err(e) => {
          tracing::error!("{:?}", e);
          response.render(Res::error("database query failed"));
        }
      }
    }
    Err(e) => {
      tracing::error!("{:?}", e);
      response.render(Res::error("json pharse failed"));
    }
  }
}

/// 用户删除
///
/// # 前端请求地址
/// `/user/delete`
///
/// # 前端请求格式
/// ```json
/// {
///   id: ... //要查询的用户id
/// }
/// ```
///
/// # 后端响应格式
/// `success` 或 `error`
///
#[handler]
async fn delete(request: &mut Request, response: &mut Response) {
  if let None = request.cookie("token") {
    tracing::info!("User not login.");
    response.render(Res::error("user not login"));
    return;
  }
  match request.parse_json::<User>().await {
    Ok(user) => {
      let dbinfo = User::delete_by_column(&db.clone(), "id", &user.id).await;
      match dbinfo {
        Ok(_) => {
          if !check_authority(
            request.cookie("token").unwrap().value().to_string(),
            user.id.unwrap(),
            Authority::Admin,
          ) {
            tracing::info!("User {} has no authority.", user.id.unwrap());
            response.render(Res::error("no authority"));
          } else {
            tracing::info!("Delete user {} successfully", &user.id.unwrap());
            response.render(Res::success());
          }
        }
        Err(e) => {
          tracing::error!("{:?}", e);
          response.render(Res::error("database query failed"));
        }
      }
    }
    Err(e) => {
      tracing::error!("{:?}", e);
      response.render(Res::error("json pharse failed"));
    }
  }
}
