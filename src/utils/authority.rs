//! 认证模块
//!
//! 将编号和权限加密为`token`，或从`token`解密为编号和权限

use jsonwebtoken::{decode, errors::Error, DecodingKey, EncodingKey, Validation};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use time::{Duration, OffsetDateTime};

use crate::utils::config::config;

/// 权限枚举
#[derive(Clone, Debug, Serialize_repr, Deserialize_repr, PartialOrd, PartialEq)]
#[repr(u16)]
pub enum Authority {
  /// 游客
  Tourist = 0,
  /// 用户
  User = 10,
  /// 评测机
  Judger = 20,
  /// 管理员
  Admin = 30,
}

#[derive(Debug, Serialize, Deserialize)]
struct JwtClaims {
  id: i32,
  auth: Authority,
  exp: i64,
}

pub struct Jwt;
impl Jwt {
  pub fn encode(id: i32, auth: Authority) -> Result<(String, OffsetDateTime), Error> {
    let exp = OffsetDateTime::now_utc() + Duration::days(config.auth.expires);
    let claim = JwtClaims {
      id: id.clone(),
      auth: auth.clone(),
      exp: exp.unix_timestamp(),
    };
    let token = jsonwebtoken::encode(
      &jsonwebtoken::Header::default(),
      &claim,
      &EncodingKey::from_secret(config.auth.secret.as_bytes()),
    )?;
    Ok((token, exp))
  }
  pub fn decode(token: String) -> Result<(i32, Authority), Error> {
    let token = decode::<JwtClaims>(
      &token,
      &DecodingKey::from_secret(config.auth.secret.as_bytes()),
      &Validation::default(),
    )?;
    Ok((token.claims.id, token.claims.auth))
  }
}

pub fn check_authority(token: String, id: i32, auth: Authority) -> bool {
  match Jwt::decode(token) {
    Ok((token_id, token_auth)) => token_id == id || token_auth >= auth,
    Err(_) => false,
  }
}
