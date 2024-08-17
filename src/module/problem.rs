use std::path::Path;

use rbatis::{crud, impl_select_page, IPage, IPageRequest, PageRequest};
use reqwest::blocking::multipart;

use salvo::{handler, Request, Response, Router};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::api::front::Res;
use crate::utils::authority::{check_authority, Authority, Jwt};
use crate::utils::config::config;
use crate::utils::db::db;
use crate::utils::error::Error;
use crate::{generate_error, handle_error};

/// 对外路由接口
pub fn init_router() -> Router {
  Router::with_path("problem")
    .push(Router::with_path("insert").post(insert))
    .push(Router::with_path("update").post(update))
    .push(Router::with_path("upload").post(upload))
    .push(Router::with_path("query").post(query))
    .push(Router::with_path("querylist").post(query_list))
    .push(Router::with_path("delete").post(delete))
}

/// 题目结构体
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Problem {
  /// 题目编号
  pid: Option<i32>,
  /// 标题
  title: Option<String>,
  /// 内容
  description: Option<String>,
  /// 测试点数量
  judge_num: Option<i32>,
  /// 时间限制, 单位：毫秒
  time_limit: Option<i32>,
  /// 内存，单位: MB
  memory_limit: Option<i32>,
  /// 上传用户
  uid: Option<i32>,
}
crud!(Problem {});

/// 题目添加
///
/// # 前端请求地址
///
/// `/problem/insert`
///
/// # 前端请求格式
///
/// ==暂定为所有登录的人都可以添加题目==
///
/// ```json5
/// {
///   "..." // problem的全部信息
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
async fn insert(request: &mut Request, response: &mut Response) {
  async fn operation(request: &mut Request, response: &mut Response) -> Result<(), Error> {
    let token: String;
    tracing::info!("{:?}", request.headers().get("Authorization"));
    match request.headers().get("Authorization") {
      Some(header) => token = String::from(header.to_str().unwrap()),
      None => return generate_error!(Error::NoToken, "Empty.".to_string()),
    }
    if !check_authority(token.clone(), 0, Authority::User) {
      return generate_error!(Error::NoAuthority, format!("Empty.").to_string());
    }
    let problem = request.parse_json::<Problem>().await?;
    if problem.title.is_none() {
      return generate_error!(Error::EmptyData, "Empty title.".to_string());
    }
    let mut problem = problem;
    problem.uid = Some(Jwt::decode(token).unwrap().0);
    if problem.judge_num.is_none() {
      problem.judge_num = Some(0);
    }
    if problem.time_limit.is_none() {
      problem.time_limit = Some(1000);
    }
    if problem.memory_limit.is_none() {
      problem.memory_limit = Some(128);
    }
    tracing::info!("{:?}", problem);
    Problem::insert(&db.clone(), &problem).await?;
    tracing::info!("Insert problem a successfully.");
    response.render(Res::success());
    Ok(())
  }
  handle_error!(operation(request, response), response);
}

/// 题目数据上传
///
/// # 前端请求地址
///
/// `/problem/upload`
///
/// # 前端请求格式
///
/// 使用 multipart/form-data
/// "pid" = 1
/// “file” = "..."
///
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
async fn upload(request: &mut Request, response: &mut Response) {
  async fn operation(request: &mut Request, response: &mut Response) -> Result<(), Error> {
    // let token: String;
    // tracing::info!("{:?}", request.headers().get("Authorization"));
    // match request.headers().get("Authorization") {
    //   Some(header) => token = String::from(header.to_str().unwrap()),
    //   None => return generate_error!(Error::NoToken, "Empty.".to_string()),
    // }
    // if !check_authority(token.clone(), 0, Authority::User) {
    //   return generate_error!(Error::NoAuthority, format!("Empty.").to_string());
    // }
    tracing::info!("{:?}", request);
    let pid = request.form::<i32>("pid").await;
    if pid.is_none() {
      return generate_error!(Error::EmptyData, "Empty pid.".to_string());
    }
    let file = request.file("file").await;
    if let None = file {
      return generate_error!(Error::EmptyData, "Empty file.".to_string());
    }
    let dest = format!("/tmp/{}.zip", pid.unwrap());
    // let dest = format!("/tmp/1.zip");
    let path = Path::new(&dest);
    if let Err(_) = std::fs::copy(file.unwrap().path(), &path) {
      return generate_error!(Error::EmptyData, "Copy file failed".to_string());
    }
    let form = multipart::Form::new().file("file", &dest);
    if form.is_err() {
      return generate_error!(Error::DataNotFound, "File not found".to_string());
    }
    let client = reqwest::blocking::Client::new();
    match client
      .post(&config.judger.url)
      .multipart(form.unwrap())
      .send()
    {
      Ok(_) => {
        response.render(Res::success());
        return Ok(());
      }
      Err(_) => {
        return generate_error!(Error::DataNotFound, "Send failed".to_string());
      }
    }
  }
  handle_error!(operation(request, response), response);
}

/// 题目修改
///
/// # 前端请求地址
///
/// `/problem/update`
///
/// # 前端请求格式
///
/// ```json5
/// {
///   "pid": 1, // 需要修改的题目编号
///   "..." // 需要修改的数据
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
async fn update(request: &mut Request, response: &mut Response) {
  async fn operation(request: &mut Request, response: &mut Response) -> Result<(), Error> {
    tracing::info!("Received a post request.",);
    let token: String;
    match request.headers().get("Authorization") {
      Some(header) => token = String::from(header.to_str().unwrap()),
      None => return generate_error!(Error::NoToken, "Empty.".to_string()),
    }
    let problem = request.parse_json::<Problem>().await?;
    if problem.pid.is_none() {
      return generate_error!(Error::EmptyData, "Empty problem id.".to_string());
    }
    let dbres = Problem::select_by_column(&db.clone(), "pid", &problem.pid).await?;
    if dbres.len() == 0 {
      return generate_error!(
        Error::DataNotFound,
        format!("Problem {}.", problem.pid.unwrap())
      );
    }
    let uid = match dbres[0].uid {
      Some(uid) => uid,
      None => 0,
    };
    if !check_authority(token, uid, Authority::Admin) {
      return generate_error!(Error::NoAuthority, "Empty".to_string());
    }
    let mut new_problem = dbres[0].clone();
    tracing::error!("{:?}", new_problem);
    if let Some(title) = problem.title {
      new_problem.title = Some(title);
    }
    if let Some(description) = problem.description {
      new_problem.description = Some(description);
    }
    if let Some(judge_num) = problem.judge_num {
      new_problem.judge_num = Some(judge_num);
    }
    if let Some(time_limit) = problem.time_limit {
      new_problem.time_limit = Some(time_limit);
    }
    if let Some(memory_limit) = problem.memory_limit {
      new_problem.memory_limit = Some(memory_limit);
    }
    if let Some(uid) = problem.uid {
      new_problem.uid = Some(uid);
    }
    let _ = Problem::update_by_column(&db.clone(), &new_problem, "pid").await?;
    tracing::info!("Update problem {} successfully.", problem.pid.unwrap());
    response.render(Res::success());
    Ok(())
  }
  handle_error!(operation(request, response), response);
}

/// 题目查询
///
/// # 前端请求地址
/// `/problem/query`
///
/// # 前端请求格式
/// ```json5
/// {
///   "pid": 1, //要查询的题目id
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
///     "...", // problem的全部信息
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
    let problem = request.parse_json::<Problem>().await?;
    if problem.pid.is_none() {
      return generate_error!(Error::EmptyData, "Empty.".to_string());
    }
    let dbres = Problem::select_by_column(&db.clone(), "pid", &problem.pid).await?;
    if dbres.len() == 0 {
      return generate_error!(
        Error::DataNotFound,
        format!("Problem {}.", problem.pid.unwrap())
      );
    }
    tracing::info!("Query problem {} successfully.", &dbres[0].pid.unwrap());
    response.render(Res::success_data(json!(&dbres[0])));
    Ok(())
  }
  handle_error!(operation(request, response), response);
}

/// 题目分页查询
///
/// # 前端请求地址
/// `/problem/querylist`
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
///         "..." // problem1
///       },
///       {
///         "..." // problem2
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
  impl_select_page!(Problem{select_page() =>"`order by pid asc`"});
  async fn operation(request: &mut Request, response: &mut Response) -> Result<(), Error> {
    tracing::info!("Received a post request.",);
    let info = request.parse_json::<Value>().await?;
    let page_no = info["page_no"].as_u64();
    let page_size = info["page_size"].as_u64();
    if page_no.is_none() || page_size.is_none() {
      return generate_error!(Error::EmptyData, "Empty.".to_string());
    }
    let dbres = Problem::select_page(
      &db.clone(),
      &PageRequest::new(page_no.unwrap(), page_size.unwrap()),
    )
    .await?;
    tracing::info!("Query {} problem(s) successfully.", dbres.records.len());
    response.render(Res::success_data(json!({
      "total": dbres.total(),
      "result": dbres.get_records()
    })));
    Ok(())
  }
  handle_error!(operation(request, response), response);
}

/// 题目删除
///
/// # 前端请求地址
/// `/problem/delete`
///
/// # 前端请求格式
/// ```json
/// {
///   pid: 1 //要删除的题目id
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
    let token: String;
    match request.headers().get("Authorization") {
      Some(header) => token = String::from(header.to_str().unwrap()),
      None => return generate_error!(Error::NoToken, "Empty.".to_string()),
    }
    let problem = request.parse_json::<Problem>().await?;
    if problem.pid.is_none() {
      return generate_error!(Error::EmptyData, "Empty.".to_string());
    }
    let dbres = Problem::select_by_column(&db.clone(), "pid", &problem.pid).await?;
    if dbres.len() == 0 {
      return generate_error!(
        Error::DataNotFound,
        format!("Problem {}.", problem.pid.unwrap())
      );
    }
    let uid = match dbres[0].uid {
      Some(uid) => uid,
      None => 0,
    };
    if !check_authority(token, uid, Authority::Admin) {
      return generate_error!(
        Error::NoAuthority,
        format!(
          "user has no authority to delete problem owned by user {}",
          uid
        )
        .to_string()
      );
    }
    Problem::delete_by_column(&db.clone(), "pid", problem.pid).await?;
    tracing::info!("Delete problem {} successfully", problem.pid.unwrap());
    response.render(Res::success());
    Ok(())
  }
  handle_error!(operation(request, response), response);
}
