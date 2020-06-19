use std::collections::HashMap;
use std::fmt;
use std::sync::Mutex;
use std::clone::Clone;
use actix_web::{Responder, web};
use bcrypt::BcryptResult;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use lazy_static::lazy_static;

use crate::json_response;

use super::user::{get_user, User};
use actix_web_httpauth::extractors::bearer::BearerAuth;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    username: String,
    email: String,
    token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthInfo {
    username: String,
    password: String,
}

#[derive(Debug)]
pub enum AuthError {
    WrongPassword,
    NotLogin,
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AuthError::WrongPassword => write!(f, "wrong password"),
            AuthError::NotLogin => write!(f, "not login"),
        }
    }
}

lazy_static! {
    pub static ref SESSION_POOL: Mutex < HashMap < String, Session > > = Mutex::new(HashMap::new());
}

fn verify_helper(user: &User, auth_info: &AuthInfo) -> BcryptResult<bool> {
    let hash = String::from_utf8(base64::decode(&user.permanent_token.replace("\n", "")).unwrap()).unwrap();
    bcrypt::verify(&auth_info.password, &hash)
}

async fn post_session_handler(req: web::Json<AuthInfo>) -> impl Responder {
    let mut session_pool = json_response!(SESSION_POOL.lock()).data.unwrap();
    let user = json_response!(get_user(None, &req.username).await);
    let user = user.data.unwrap();
    if json_response!(verify_helper(&user, &req)).data.unwrap() {
        let uuid = Uuid::new_v4().to_string();
        let session = Session{
            email: user.email.clone(),
            username: user.username.clone(),
            token: uuid.clone()
        };
        session_pool.insert(uuid, session.clone());
        web::Json(json_response!(Ok::<Session, AuthError>(session)))
    } else {
        web::Json(json_response!(Err::<Session, AuthError>(AuthError::WrongPassword)))
    }
}

async fn get_session_handler(req: BearerAuth) -> impl Responder {
    let session_pool = json_response!(SESSION_POOL.lock()).data.unwrap();
    match session_pool.get(req.token()) {
        Some(session) => web::Json(json_response!(Ok::<Session, AuthError>(session.clone()))),
        None => web::Json(json_response!(Err::<Session, AuthError>(AuthError::NotLogin))),
    }
}

async fn delete_session_handler(req: BearerAuth) -> impl Responder {
    let mut session_pool = json_response!(SESSION_POOL.lock()).data.unwrap();
    match session_pool.remove(req.token()) {
        Some(session) => web::Json(json_response!(Ok::<Session, AuthError>(session.clone()))),
        None => web::Json(json_response!(Err::<Session, AuthError>(AuthError::NotLogin))),
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/session")
            .route(web::post().to(post_session_handler))
            .route(web::get().to(get_session_handler))
            .route(web::delete().to(delete_session_handler))
    );
}