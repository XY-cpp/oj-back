use rbatis::snowflake::new_snowflake_id;
use rbatis::{crud, rbdc::datetime::DateTime};
use rbdc_mysql::protocol::auth;
use salvo::http::cookie::Cookie;
use salvo::{handler, Request, Response, Router};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::api::front::Res;
use crate::utils::authority::{self, check_authority, Authority, Jwt};
use crate::utils::db::db;
use crate::utils::error::Error;
use crate::{generate_error, handle_error};

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
  uid: Option<i32>,
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
///
/// `/user/register`
///
/// # 前端请求格式
///
/// ```json5
/// {
///   "accout": "...",
///   "password": "..."
/// }
/// ```
///
/// # 后端响应格式
///
/// - 成功
/// ```json5
/// {
///   "status": "success"
/// }
/// ```
///
/// - 失败
/// ``` json5
/// {
///   "status": "error",
///   "message": "data error", // 或 "internal error"
///   "data": "..." // 出错数据
/// }
/// ```
///
#[handler]
async fn register(request: &mut Request, response: &mut Response) {
  async fn operation(request: &mut Request, response: &mut Response) -> Result<(), Error> {
    tracing::info!("Received a request to register a user.",);
    let user = request.parse_json::<User>().await?;
    if user.account.is_none() || user.password.is_none() {
      return generate_error!(
        Error::WrongDataFormat,
        "empty username or passowrd".to_string()
      );
    }
    let dbres = User::select_by_column(&db.clone(), "account", &user.account).await?;
    if dbres.len() > 0 {
      return generate_error!(
        Error::DuplicateData,
        format!("account: {}", &user.account.unwrap()).to_string()
      );
    }
    let mut user = user;
    user.avatar = Some(String::from("http://127.0.0.1:8001/null"));
    user.join_time = Some(DateTime::now());
    user.auth = Some(Authority::User);
    let dbinfo = User::insert(&db.clone(), &user).await?;
    tracing::info!("{}", dbinfo);
    response.render(Res::success());
    Ok(())
  }
  handle_error!(operation(request, response), response);
}

/// 用户登录
///
/// # 前端请求地址
///
/// `/user/login`
///
/// # 前端请求格式
///
/// ```json5
/// {
///   "accout": "...",
///   "password": "..."
/// }
/// ```
///
/// # 后端响应格式
///
/// - 成功
/// ```json5
/// {
///   "status": "success",
///   "data": [
///     "...", //user的全部信息
///   ]
/// }
/// ```
///
/// - 失败
/// ``` json5
/// {
///   "status": "error",
///   "message": "data error", // 或 "internal error"
///   "data": "..." // 出错数据
/// }
/// ```
///
#[handler]
async fn login(request: &mut Request, response: &mut Response) {
  async fn operation(request: &mut Request, response: &mut Response) -> Result<(), Error> {
    tracing::info!("Received a request to login.",);
    let user = request.parse_json::<User>().await?;
    if user.account.is_none() || user.password.is_none() {
      return generate_error!(
        Error::WrongDataFormat,
        "empty username or passowrd".to_string()
      );
    }
    let dbres = User::select_by_column(&db.clone(), "account", &user.account).await?;
    if dbres.len() == 0 {
      return generate_error!(
        Error::DataNotFound,
        format!("account: {}", &user.account.unwrap()).to_string()
      );
    } else if user.password != dbres[0].password {
      return generate_error!(
        Error::WrongPassword,
        format!("account: {}", &user.account.unwrap()).to_string()
      );
    }
    let user = dbres[0].clone();
    let (token, exp) = Jwt::encode(user.uid.unwrap(), user.auth.clone().unwrap())?;
    tracing::info!(
      "User {} login successfully with token {}.",
      user.uid.unwrap(),
      &token
    );
    let mut cookie = Cookie::new("token", token);
    cookie.set_expires(exp);
    response.add_cookie(cookie);
    response.render(Res::success_data(json!(&user)));
    Ok(())
  }
  handle_error!(operation(request, response), response);
}

/// 用户修改
///
/// # 前端请求地址
///
/// `/user/login`
///
/// # 前端请求格式
///
/// ```json5
/// {
///   "id": [num], // 要修改的用户编号
///   "...", //需要修改的数据
/// }
/// ```
///
/// # 后端响应格式
///
/// - 成功
/// ```json5
/// {
///   "status": "success",
/// }
/// ```
///
/// - 失败
/// ``` json5
/// {
///   "status": "error",
///   "message": "data error", // 或 "internal error"
///   "data": "..." // 出错数据
/// }
/// ```
///
#[handler]
async fn update(request: &mut Request, response: &mut Response) {
  async fn operation(request: &mut Request, response: &mut Response) -> Result<(), Error> {
    tracing::info!("Received a request to update a user.",);
    if let None = request.cookie("token") {
      return generate_error!(Error::NoAuthority, "user not login".to_string());
    }
    let user = request.parse_json::<User>().await?;
    if user.uid.is_none() {
      return generate_error!(Error::WrongDataFormat, "id not found".to_string());
    }
    if !check_authority(
      request.cookie("token").unwrap().value().to_string(),
      user.uid.unwrap(),
      Authority::Admin,
    ) {
      return generate_error!(
        Error::NoAuthority,
        format!("user has no authority to update user {}", user.uid.unwrap()).to_string()
      );
    }
    let dbres = User::select_by_column(&db.clone(), "id", &user.uid).await?;
    if dbres.len() == 0 {
      return generate_error!(Error::DataNotFound, user.uid.unwrap().to_string());
    }
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
      if new_user.auth.clone().unwrap() < authority {
        return generate_error!(
          Error::NoAuthority,
          format!(
            "user {} has no authority to get higher authority",
            new_user.uid.unwrap()
          )
          .to_string()
        );
      }
      new_user.auth = Some(authority);
    }
    let dbinfo = User::update_by_column(&db.clone(), &new_user, "id").await?;
    tracing::info!("{}", dbinfo);
    response.render(Res::success());
    Ok(())
  }
  handle_error!(operation(request, response), response);
}

/// 用户查询
///
/// # 前端请求地址
/// `/user/query`
///
/// # 前端请求格式
/// ```json5
/// {
///   "id": [num], //要查询的用户id
/// }
/// ```
///
/// # 后端响应格式
///
/// - 成功
/// ```json5
/// {
///   "status": "success",
///   "data": [
///     "...", //user的全部信息
///   ]
/// }
/// ```
///
/// - 失败
/// ``` json5
/// {
///   "status": "error",
///   "message": "data error", // 或 "internal error"
///   "data": "..." // 出错数据
/// }
/// ```
///
#[handler]
async fn query(request: &mut Request, response: &mut Response) {
  async fn operation(request: &mut Request, response: &mut Response) -> Result<(), Error> {
    tracing::info!("Received a request to query a user.",);
    let user = request.parse_json::<User>().await?;
    if user.uid.is_none() {
      return generate_error!(Error::WrongDataFormat, "".to_string());
    }
    let dbres = User::select_by_column(&db.clone(), "id", &user.uid).await?;
    if dbres.len() == 0 {
      return generate_error!(
        Error::DataNotFound,
        format!("id: {}", &user.uid.unwrap()).to_string()
      );
    } else {
      tracing::info!("Query user {} successfully.", &dbres[0].uid.unwrap());
      response.render(Res::success_data(json!(&dbres[0])));
    }
    Ok(())
  }
  handle_error!(operation(request, response), response);
}

/// 用户删除
///
/// # 前端请求地址
/// `/user/delete`
///
/// # 前端请求格式
/// ```json
/// {
///   id: ... //要删除的用户id
/// }
/// ```
///
/// # 后端响应格式
///
/// - 成功
/// ```json5
/// {
///   "status": "success",
/// }
/// ```
///
/// - 失败
/// ``` json5
/// {
///   "status": "error",
///   "message": "data error", // 或 "internal error"
///   "data": "..." // 出错数据
/// }
/// ```
#[handler]
async fn delete(request: &mut Request, response: &mut Response) {
  async fn operation(request: &mut Request, response: &mut Response) -> Result<(), Error> {
    tracing::info!("Received a request to delete a user.",);
    if let None = request.cookie("token") {
      return generate_error!(Error::NoAuthority, "user not login".to_string());
    }
    let user = request.parse_json::<User>().await?;
    if user.uid.is_none() {
      return generate_error!(Error::WrongDataFormat, "".to_string());
    }
    if !check_authority(
      request.cookie("token").unwrap().value().to_string(),
      user.uid.unwrap(),
      Authority::Admin,
    ) {
      return generate_error!(
        Error::NoAuthority,
        format!("user has no authority to delete user {}", user.uid.unwrap()).to_string()
      );
    }
    let _ = User::delete_by_column(&db.clone(), "id", &user.uid).await?;
    tracing::info!("Delete user {} successfully", &user.uid.unwrap());
    response.render(Res::success());
    Ok(())
  }
  handle_error!(operation(request, response), response);
}
