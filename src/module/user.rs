use rbatis::{crud, rbdc::datetime::DateTime};
use rbatis::{impl_select_page, IPage, IPageRequest, PageRequest};
use salvo::http::cookie::Cookie;
use salvo::{handler, Request, Response, Router};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::api::front::Res;
use crate::utils::authority::{check_authority, Authority, Jwt};
use crate::utils::db::db;
use crate::utils::error::Error;
use crate::{generate_error, handle_error};

/// 对外路由接口
pub fn init_router() -> Router {
  Router::with_path("user")
    .push(Router::with_path("register").post(register))
    .push(Router::with_path("login").post(login))
    .push(Router::with_path("tokenlogin").get(tokenlogin))
    .push(Router::with_path("update").post(update))
    .push(Router::with_path("query").post(query))
    .push(Router::with_path("querylist").post(query_list))
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
///   "message": "...", // 错误类型见error.rs
///   "data": "..." // 具体出错信息
/// }
/// ```
///
#[handler]
async fn register(request: &mut Request, response: &mut Response) {
  async fn operation(request: &mut Request, response: &mut Response) -> Result<(), Error> {
    tracing::info!("Received a post request.",);
    let user = request.parse_json::<User>().await?;
    if user.account.is_none()
      || user.password.is_none()
      || user.account.clone().unwrap().is_empty()
      || user.password.clone().unwrap().is_empty()
    {
      return generate_error!(Error::EmptyData, "Empty username or passowrd.".to_string());
    }
    let dbres = User::select_by_column(&db.clone(), "account", &user.account).await?;
    if dbres.len() > 0 {
      return generate_error!(
        Error::DuplicateData,
        format!("account: {}.", &user.account.unwrap()).to_string()
      );
    }
    let mut user = user;
    user.avatar = Some(String::from("http://127.0.0.1:8001/null"));
    user.join_time = Some(DateTime::now());
    user.auth = Some(Authority::User);
    User::insert(&db.clone(), &user).await?;
    tracing::info!("Register successfully");
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
///   "message": "...", // 错误类型见error.rs
///   "data": "..." // 具体出错信息
/// }
/// ```
///
#[handler]
async fn login(request: &mut Request, response: &mut Response) {
  async fn operation(request: &mut Request, response: &mut Response) -> Result<(), Error> {
    tracing::info!("Received a post request",);
    let user = request.parse_json::<User>().await?;
    if user.account.is_none() || user.password.is_none() {
      return generate_error!(Error::EmptyData, "Empty username or passowrd.".to_string());
    }
    let dbres = User::select_by_column(&db.clone(), "account", &user.account).await?;
    if dbres.len() == 0 || user.password != dbres[0].password {
      return generate_error!(
        Error::WrongPassword,
        format!(
          "account: {}, password: {}",
          user.account.unwrap(),
          user.password.unwrap()
        )
        .to_string()
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

/// 用户token登录
///
/// # 前端请求地址
///
/// `/user/tokenlogin`
///
/// # 前端请求格式
///
/// 使用`get`请求
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
///   "message": "...", // 错误类型见error.rs
///   "data": "..." // 具体出错信息
/// }
/// ```
///
#[handler]
async fn tokenlogin(request: &mut Request, response: &mut Response) {
  async fn operation(request: &mut Request, response: &mut Response) -> Result<(), Error> {
    tracing::info!("Received a post request",);
    match request.cookie("token") {
      Some(cookie) => match Jwt::decode(cookie.value().to_string()) {
        Ok((uid, _)) => {
          let dbres = User::select_by_column(&db.clone(), "uid", uid).await?;
          if dbres.len() == 0 {
            return generate_error!(Error::DataNotFound, format!("User {}.", uid));
          }
          tracing::info!(
            "User {} login successfully with token {}.",
            uid,
            cookie.value().to_string()
          );
          response.render(Res::success_data(json!(&dbres[0])));
          return Ok(());
        }
        Err(_) => {
          return generate_error!(Error::NoToken, "Token is wrong or expired.".to_string());
        }
      },
      None => {
        return generate_error!(Error::NoToken, "Empty.".to_string());
      }
    }
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
///   "uid": 1, // 要修改的用户编号
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
///   "message": "...", // 错误类型见error.rs
///   "data": "..." // 具体出错信息
/// }
/// ```
///
#[handler]
async fn update(request: &mut Request, response: &mut Response) {
  async fn operation(request: &mut Request, response: &mut Response) -> Result<(), Error> {
    tracing::info!("Received a post request.",);
    if let None = request.cookie("token") {
      return generate_error!(Error::NoToken, "Empty.".to_string());
    }
    let user = request.parse_json::<User>().await?;
    if user.uid.is_none() {
      return generate_error!(Error::EmptyData, "Uid not found.".to_string());
    }
    if !check_authority(
      request.cookie("token").unwrap().value().to_string(),
      user.uid.unwrap(),
      Authority::Admin,
    ) {
      return generate_error!(
        Error::NoAuthority,
        format!("User {}.", user.uid.unwrap()).to_string()
      );
    }
    let dbres = User::select_by_column(&db.clone(), "uid", &user.uid).await?;
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
    // if let Some(join_time) = user.join_time {
    //   new_user.join_time = Some(join_time);
    // }
    if let Some(authority) = user.auth {
      if new_user.auth.clone().unwrap() < authority {
        return generate_error!(
          Error::NoAuthority,
          format!("User {}.", new_user.uid.unwrap()).to_string()
        );
      }
      new_user.auth = Some(authority);
    }
    let _ = User::update_by_column(&db.clone(), &new_user, "uid").await?;
    tracing::info!("Update user {} successfully.", user.uid.unwrap());
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
///   "uid": 1, //要查询的用户id
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
///   "message": "...", // 错误类型见error.rs
///   "data": "..." // 具体出错信息
/// }
/// ```
///
#[handler]
async fn query(request: &mut Request, response: &mut Response) {
  async fn operation(request: &mut Request, response: &mut Response) -> Result<(), Error> {
    tracing::info!("Received a post request.",);
    let user = request.parse_json::<User>().await?;
    if user.uid.is_none() {
      return generate_error!(Error::EmptyData, "Empty.".to_string());
    }
    let dbres = User::select_by_column(&db.clone(), "uid", &user.uid).await?;
    if dbres.len() == 0 {
      return generate_error!(Error::DataNotFound, user.uid.unwrap().to_string());
    } else {
      tracing::info!("Query user {} successfully.", &dbres[0].uid.unwrap());
      response.render(Res::success_data(json!(&dbres[0])));
    }
    Ok(())
  }
  handle_error!(operation(request, response), response);
}

/// 用户分页查询
///
/// # 前端请求地址
/// `/user/querylist`
///
/// # 前端请求格式
/// ```json5
/// {
///   page_no: 1, // 页号
///   page_size: 10, // 页长
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
///     "total": 2, // 总的记录数（不是当前页的)
///     "result": [
///       {
///         "..." // user1
///       }
///       {
///         "..." // user2
///       }
///     ]
///   ]
/// }
/// ```
///
/// - 失败
/// ``` json5
/// {
///   "status": "error",
///   "message": "...", // 错误类型见error.rs
///   "data": "..." // 具体出错信息
/// }
/// ```
///
#[handler]
async fn query_list(request: &mut Request, response: &mut Response) {
  impl_select_page!(User{select_page() =>"`order by uid asc`"});
  async fn operation(request: &mut Request, response: &mut Response) -> Result<(), Error> {
    tracing::info!("Received a post request.",);
    let info = request.parse_json::<Value>().await?;
    let page_no = info["page_no"].as_u64();
    let page_size = info["page_size"].as_u64();
    if page_no.is_none() || page_size.is_none() {
      return generate_error!(Error::EmptyData, "Empty.".to_string());
    }
    let dbres = User::select_page(
      &db.clone(),
      &PageRequest::new(page_no.unwrap(), page_size.unwrap()),
    )
    .await?;
    tracing::info!("Query {} user(s) successfully.", dbres.records.len());
    response.render(Res::success_data(json!({
      "total": dbres.total(),
      "result": dbres.get_records()
    })));
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
///   uid: 1 //要删除的用户id
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
///   "message": "...", // 错误类型见error.rs
///   "data": "..." // 具体出错信息
/// }
/// ```
#[handler]
async fn delete(request: &mut Request, response: &mut Response) {
  async fn operation(request: &mut Request, response: &mut Response) -> Result<(), Error> {
    tracing::info!("Received a post request.",);
    if let None = request.cookie("token") {
      return generate_error!(Error::NoToken, "Empty.".to_string());
    }
    let user = request.parse_json::<User>().await?;
    if user.uid.is_none() {
      return generate_error!(Error::EmptyData, "".to_string());
    }
    if !check_authority(
      request.cookie("token").unwrap().value().to_string(),
      user.uid.unwrap(),
      Authority::Admin,
    ) {
      return generate_error!(
        Error::NoAuthority,
        format!("User {}.", user.uid.unwrap()).to_string()
      );
    }
    let _ = User::delete_by_column(&db.clone(), "uid", &user.uid).await?;
    tracing::info!("Delete user {} successfully.", &user.uid.unwrap());
    response.render(Res::success());
    Ok(())
  }
  handle_error!(operation(request, response), response);
}
