use rbatis::crud;
use rbatis::rbdc::DateTime;
use salvo::{handler, Request, Response, Router};
use serde::{Deserialize, Serialize};
use serde_json::json;
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
}

/// 评测记录结构体
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Record {
  /// 评测编号
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
  /// 运行时间
  run_time: Option<f32>,
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
    match request.cookie("token") {
      Some(token) => {
        if !check_authority(token.value().to_string(), 0, Authority::Judger) {
          return generate_error!(Error::NoToken, "Empty.".to_string());
        }
      }
      None => {
        return generate_error!(Error::NoToken, "Empty.".to_string());
      }
    }
    let record = request.parse_json::<Record>().await?;
    if record.uid.is_none() || record.pid.is_none() || record.language.is_none() {
      return generate_error!(Error::EmptyData, "Empty.".to_string());
    }
    let mut record = record;
    record.submit_time = Some(DateTime::now());
    record.status = Some(Status::Waiting);
    let dbinfo = Record::insert(&db.clone(), &record).await?;
    tracing::info!("{}", dbinfo);
    response.render(Res::success());
    Ok(())
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
    if let None = request.cookie("token") {
      return generate_error!(Error::NoToken, "Empty.".to_string());
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
    if !check_authority(
      request.cookie("token").unwrap().value().to_string(),
      0,
      Authority::Judger,
    ) {
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
/// ```json5
/// {
///   "rid": 1, //要查询的题目id
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
///     "...", // record的全部信息
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
    let record = request.parse_json::<Record>().await?;
    if record.rid.is_none() {
      return generate_error!(Error::EmptyData, "Empty.".to_string());
    }
    let dbres = Record::select_by_column(&db.clone(), "rid", &record.rid).await?;
    if dbres.len() == 0 {
      return generate_error!(Error::DataNotFound, record.rid.unwrap().to_string());
    }
    tracing::info!("Query record {} successfully", record.rid.unwrap());
    response.render(Res::success_data(json!(&dbres[0])));
    Ok(())
  }
  handle_error!(operation(request, response), response);
}
