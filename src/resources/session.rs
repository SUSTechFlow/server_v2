use lazy_static::lazy_static;
use std::sync::Mutex;
use std::collections::HashMap;
use super::user::{get_user, User};
use actix_web::{web, HttpResponse, Responder};
use serde::{Serialize, Deserialize};
use actix_web::body::Body;
use bcrypt::BcryptResult;
use uuid::Uuid;
use crate::json_response;
use std::fmt;
lazy_static! {
    static ref SESSION_POOL: Mutex<HashMap<String, User>> = Mutex::new(HashMap::new());
}
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthInfo {
    username: String,
    password: String,
}

#[derive(Debug)]
pub enum AuthError {
    WrongPassword,
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "wrong password")
    }
}

fn verify_helper(user: &User, auth_info: &AuthInfo) -> BcryptResult<bool> {
    let hash = String::from_utf8(base64::decode(&user.permanent_token.replace("\n", "")).unwrap()).unwrap();
    bcrypt::verify(&auth_info.password, &hash)
}

pub async fn post_session_handler(req: web::Json<AuthInfo>) -> impl Responder {
    let mut session_pool = json_response!(SESSION_POOL.lock()).data.unwrap();
    let user = json_response!(get_user(None, &req.username).await);
    let user = user.data.unwrap();
    if json_response!(verify_helper(&user, &req)).data.unwrap() {
        let uuid = Uuid::new_v4().to_string();
        session_pool.insert(uuid.clone(), user);
        web::Json(json_response!(Ok::<String, AuthError>(uuid)))
    } else {
        web::Json(json_response!(Err::<String, AuthError>(AuthError::WrongPassword)))
    }

}
