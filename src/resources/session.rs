use std::clone::Clone;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::sync::Mutex;

use actix_web::{Responder, web};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::json_response;
use crate::util::crypto::verify_helper;

use super::user::get_user;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub(crate) username: String,
    pub(crate) email: String,
    token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthInfo {
    pub(crate) username: String,
    pub(crate) password: String,
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

impl Error for AuthError {}

lazy_static! {
    pub static ref SESSION_POOL: Mutex < HashMap < String, Session > > = Mutex::new(HashMap::new());
}



pub(crate) async fn get_session(auth: BearerAuth) -> Result<Session, Box<dyn Error>> {
    let session_pool = SESSION_POOL.lock()?;
    match session_pool.get(auth.token()) {
        Some(session) => Ok::<Session, Box<dyn Error>>(session.clone()),
        None => Err::<Session, Box<dyn Error>>(Box::new(AuthError::NotLogin)),
    }
}

pub(crate) async fn post_session(auth: AuthInfo) -> Result<Session, Box<dyn Error>> {
    let mut session_pool = SESSION_POOL.lock()?;
    let user = get_user(None, &auth.username).await?;
    if verify_helper(&user.permanent_token, &auth.password) {
        let uuid = Uuid::new_v4().to_string();
        let session = Session {
            email: user.email.clone(),
            username: user.username.clone(),
            token: uuid.clone(),
        };
        session_pool.insert(uuid, session.clone());
        Ok(session)
    } else {
        Err(Box::new(AuthError::WrongPassword))
    }
}

async fn delete_session(req: BearerAuth) -> Result<Session, Box<dyn Error>> {
    let mut session_pool = SESSION_POOL.lock()?;
    match session_pool.remove(req.token()) {
        Some(session) => Ok::<Session, Box<dyn Error>>(session.clone()),
        None => Err::<Session, Box<dyn Error>>(Box::new(AuthError::NotLogin)),
    }
}

async fn delete_session_handler(req: BearerAuth) -> impl Responder {
    web::Json(json_response!(delete_session(req).await))
}

async fn post_session_handler(auth: web::Json<AuthInfo>) -> impl Responder {
    web::Json(json_response!(post_session(auth.0).await))
}

async fn get_session_handler(auth: BearerAuth) -> impl Responder {
    web::Json(json_response!(get_session(auth).await))
}


pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/session")
            .route(web::post().to(post_session_handler))
            .route(web::get().to(get_session_handler))
            .route(web::delete().to(delete_session_handler))
    );
}