use rbatis::crud;
use salvo::{handler, Request, Response, Router};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::api::front::Res;
use crate::utils::authority::{check_authority, Authority, Jwt};
use crate::utils::db::db;
use crate::utils::error::Error;
use crate::{generate_error, handle_error};

/// 对外路由接口
pub fn init_router() -> Router {
  Router::with_path("problem")
    .push(Router::with_path("insert").post(insert))
    .push(Router::with_path("update").post(update))
    .push(Router::with_path("query").post(query))
    .push(Router::with_path("delete").post(delete))
}

/// 题目结构体
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Problem {
  /// 题目编号
  id: Option<i32>,
  /// 标题
  title: Option<String>,
  /// 内容
  description: Option<String>,
  /// 测试点数量
  judge_num: Option<i32>,
  /// 时间限制，单位：s
  time_limit: Option<f32>,
  /// 内存，单位: KB
  memory_limit: Option<i32>,
  /// 上传用户
  user_id: Option<i32>,
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
///   "message": "data error", // 或 "internal error"
///   "data": "..." // 出错数据
/// }
/// ```
///
#[handler]
async fn insert(request: &mut Request, response: &mut Response) {
  async fn operation(request: &mut Request, response: &mut Response) -> Result<(), Error> {
    tracing::info!("Received a request to insert.",);
    match request.cookie("token") {
      Some(token) => {
        if !check_authority(token.value().to_string(), 0, Authority::User) {
          return generate_error!(
            Error::NoAuthority,
            format!("user has no authority to insert a problem").to_string()
          );
        }
      }
      None => {
        return generate_error!(Error::NoAuthority, "user not login".to_string());
      }
    }
    let problem = request.parse_json::<Problem>().await?;
    if problem.title.is_none() {
      return generate_error!(Error::WrongDataFormat, "empty title".to_string());
    }
    let mut problem = problem;
    problem.user_id = Some(
      Jwt::decode(request.cookie("token").unwrap().value().to_string())
        .unwrap()
        .0,
    );
    let dbinfo = Problem::insert(&db.clone(), &problem).await?;
    tracing::info!("{}", dbinfo);
    response.render(Res::success());
    Ok(())
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
///   "id": [num], // 需要修改的题目编号
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
    let problem = request.parse_json::<Problem>().await?;
    if problem.id.is_none() {
      return generate_error!(Error::WrongDataFormat, "id not found".to_string());
    }
    let dbres = Problem::select_by_column(&db.clone(), "id", &problem.id).await?;
    if dbres.len() == 0 {
      return generate_error!(Error::DataNotFound, problem.id.unwrap().to_string());
    }
    let user_id = match dbres[0].user_id {
      Some(user_id) => user_id,
      None => 0,
    };
    if !check_authority(
      request.cookie("token").unwrap().value().to_string(),
      user_id,
      Authority::Admin,
    ) {
      return generate_error!(
        Error::NoAuthority,
        format!(
          "user has no authority to update problem owned by {}",
          user_id
        )
        .to_string()
      );
    }
    let mut new_problem = dbres[0].clone();
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
    if let Some(user_id) = problem.user_id {
      new_problem.user_id = Some(user_id);
    }
    let dbinfo = Problem::update_by_column(&db.clone(), &new_problem, "id").await?;
    tracing::info!("{}", dbinfo);
    response.render(Res::success());
    Ok(())
  }
  handle_error!(operation(request, response), response);
}

/// 题目查询
///
/// # 前端请求地址
/// `/user/query`
///
/// # 前端请求格式
/// ```json5
/// {
///   "id": [num], //要查询的题目id
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
///   "message": "data error", // 或 "internal error"
///   "data": "..." // 出错数据
/// }
/// ```
///
#[handler]
async fn query(request: &mut Request, response: &mut Response) {
  async fn operation(request: &mut Request, response: &mut Response) -> Result<(), Error> {
    tracing::info!("Received a request to query.",);
    let problem = request.parse_json::<Problem>().await?;
    if problem.id.is_none() {
      return generate_error!(Error::WrongDataFormat, "".to_string());
    }
    let dbres = Problem::select_by_column(&db.clone(), "id", &problem.id).await?;
    if dbres.len() == 0 {
      return generate_error!(
        Error::DataNotFound,
        format!("account: {}", &problem.id.unwrap()).to_string()
      );
    } else {
      tracing::info!("Query problem {} successfully.", &dbres[0].id.unwrap());
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
    let problem = request.parse_json::<Problem>().await?;
    if problem.id.is_none() {
      return generate_error!(Error::WrongDataFormat, "".to_string());
    }
    let dbres = Problem::select_by_column(&db.clone(), "id", &problem.id).await?;
    if dbres.len() == 0 {
      return generate_error!(Error::DataNotFound, problem.id.unwrap().to_string());
    }
    let user_id = match dbres[0].user_id {
      Some(user_id) => user_id,
      None => 0,
    };
    if !check_authority(
      request.cookie("token").unwrap().value().to_string(),
      user_id,
      Authority::Admin,
    ) {
      return generate_error!(
        Error::NoAuthority,
        format!(
          "user has no authority to delete problem owned by user {}",
          user_id
        )
        .to_string()
      );
    }
    let _ = Problem::delete_by_column(&db.clone(), "id", problem.id).await?;
    tracing::info!("Delete user {} successfully", &problem.id.unwrap());
    response.render(Res::success());
    Ok(())
  }
  handle_error!(operation(request, response), response);
}
