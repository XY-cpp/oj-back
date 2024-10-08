use crate::utils::config::config;
use rbatis::rbdc::DateTime;
use rbatis::{crud, impl_select, impl_select_page, IPage, IPageRequest, PageRequest};
use reqwest::Client;
use salvo::{handler, Request, Response, Router};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::api::front::Res;
use crate::utils::authority::{check_authority, Authority};
use crate::utils::db::db;
use crate::utils::error::Error;
use crate::{generate_error, handle_error};

/// 对外路由接口
pub fn init_router() -> Router {
  Router::with_path("record")
    .push(Router::with_path("insert").post(insert))
    .push(Router::with_path("update").post(update))
    .push(Router::with_path("query").post(query))
    .push(Router::with_path("querylist").post(query_list))
}

/// 评测记录结构体
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Record {
  /// 评测编
  rid: Option<i32>,
  /// 用户编号
  uid: Option<i32>,
  /// 题目编号
  pid: Option<i32>,
  /// 语言
  language: Option<Language>,
  /// 代码
  code: Option<String>,
  /// 提交时间
  submit_time: Option<DateTime>,
  /// 评测状态
  status: Option<Status>,
  /// 运行时间, 单位：毫秒
  run_time: Option<i32>,
}
crud!(Record {});

/// 运行状态枚举
#[derive(Clone, Debug, Serialize_repr, Deserialize_repr)]
#[repr(u16)]
enum Status {
  /// 等待评测
  Waiting = 10,
  /// 评测中
  Pending = 20,
  /// 通过
  Ac = 30,
  /// 答案错误
  Wa = 40,
  /// 运行错误
  Re = 50,
  /// 空间超限
  Mle = 60,
  /// 时间超限
  Tle = 70,
  /// 编译错误
  Ce = 80,
  /// 位置错误
  Uke = 90,
}

/// 语言枚举类
#[derive(Clone, Debug, Serialize_repr, Deserialize_repr)]
#[repr(u16)]
enum Language {
  C = 10,
  Cpp = 20,
  Python3 = 30,
  Rust = 40,
}

/// 评测记录添加
///
/// # 前端请求地址
///
/// `/record/insert`
///
/// # 前端请求格式
///
/// ```json5
/// {
///   "uid": 1 // 用户编号
///   "pid": 1 // 题目编号
///   "language": 10 // 语言类型
///   "code": "..." // 代码
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
    tracing::info!("Received a post request.",);
    let token: String;
    match request.headers().get("Authorization") {
      Some(header) => token = String::from(header.to_str().unwrap()),
      None => return generate_error!(Error::NoToken, "Empty.".to_string()),
    }
    if !check_authority(token, 0, Authority::User) {
      return generate_error!(Error::NoToken, "Empty.".to_string());
    }
    let record = request.parse_json::<Record>().await?;
    if record.uid.is_none() || record.pid.is_none() || record.language.is_none() {
      return generate_error!(Error::EmptyData, "Empty.".to_string());
    }
    let mut record = record;
    record.submit_time = Some(DateTime::now());
    record.status = Some(Status::Waiting);
    record.rid = Some(
      Record::insert(&db.clone(), &record)
        .await?
        .last_insert_id
        .as_i64()
        .unwrap() as i32,
    );
    tracing::info!("Insert a record successfully");
    response.render(Res::success());

    let client = Client::new();
    match client
      .post(&config.judger.url)
      .json(&json!({
        "pid": record.pid,
        "rid": record.rid,
        "code": record.code,
        "language": record.language,
        "opt": []
      }))
      .send()
      .await
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

/// 评测记录修改
///
/// # 前端请求地址
///
/// `/record/update`
///
/// # 前端请求格式
///
/// 只能由评测机来进行修改状态和运行时间
///
/// ```json5
/// {
///   "rid": 1, // 需要修改的题目编号
///   "status": "...",
///   "runtime": "..."
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
    let record = request.parse_json::<Record>().await?;
    if record.rid.is_none() {
      return generate_error!(Error::EmptyData, "Empty.".to_string());
    }
    let dbres = Record::select_by_column(&db.clone(), "rid", &record.rid).await?;
    if dbres.len() == 0 {
      return generate_error!(
        Error::DataNotFound,
        format!("Record {}.", record.rid.unwrap()).to_string()
      );
    }
    if !check_authority(token, 0, Authority::Judger) {
      return generate_error!(Error::NoAuthority, "Empty.".to_string());
    }
    let mut new_record = dbres[0].clone();
    if let Some(status) = record.status {
      new_record.status = Some(status);
    }
    if let Some(run_time) = record.run_time {
      new_record.run_time = Some(run_time);
    }
    let _ = Record::update_by_column(&db.clone(), &new_record, "rid").await?;
    tracing::info!("Update record {} successfully", record.rid.unwrap());
    response.render(Res::success());
    Ok(())
  }
  handle_error!(operation(request, response), response);
}

/// 记录查询
///
/// # 前端请求地址
/// `/user/query`
///
/// # 前端请求格式
///
/// 什么都不填写返回全部记录
///
/// ```json5
/// {
///   "rid": 1,
///   "uid": 1,
///   "pid": 1,
///   "language": 10,
///   "status": 10
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
///     {
///       "..." // "record1"记录
///     },
///     {
///       "..." // "record2"记录
///     }
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
  impl_select!(Record{
    select_by_ids(
      rid:Option<i32>,
      uid:Option<i32>,
      pid:Option<i32>
      language:Option<Language>,
      status:Option<Status>
    ) => "
    `where (rid = #{rid} or #{rid} is null)`
    ` and (uid = #{uid} or #{uid} is null)`
    ` and (pid = #{pid} or #{pid} is null)`
    ` and (language = #{language} or #{language} is null)`
    ` and (status = #{status} or #{status} is null)`
    `order by rid desc;`
  "
  });
  async fn operation(request: &mut Request, response: &mut Response) -> Result<(), Error> {
    tracing::info!("Received a post request.",);
    let record = request.parse_json::<Record>().await?;
    let dbres = Record::select_by_ids(
      &db.clone(),
      record.rid,
      record.uid,
      record.pid,
      record.language,
      record.status,
    )
    .await?;
    tracing::info!("Query {} record(s) successfully", dbres.len());
    response.render(Res::success_data(json!(dbres)));
    Ok(())
  }
  handle_error!(operation(request, response), response);
}

/// 记录分页查询
///
/// # 前端请求地址
/// `/record/querylist`
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
///         "..." // record1
///       },
///       {
///         "..." // record2
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
  impl_select_page!(Record{select_page() =>"`order by rid desc`"});
  async fn operation(request: &mut Request, response: &mut Response) -> Result<(), Error> {
    tracing::info!("Received a post request.",);
    let info = request.parse_json::<Value>().await?;
    let page_no = info["page_no"].as_u64();
    let page_size = info["page_size"].as_u64();
    if page_no.is_none() || page_size.is_none() {
      return generate_error!(Error::EmptyData, "Empty.".to_string());
    }
    let dbres = Record::select_page(
      &db.clone(),
      &PageRequest::new(page_no.unwrap(), page_size.unwrap()),
    )
    .await?;
    tracing::info!("Query {} record(s) successfully.", dbres.records.len());
    response.render(Res::success_data(json!({
      "total": dbres.total(),
      "result": dbres.get_records()
    })));
    Ok(())
  }
  handle_error!(operation(request, response), response);
}
