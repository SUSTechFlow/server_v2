use std::clone::Clone;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::sync::Mutex;

use actix_web::{Responder, web};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use chrono::{DateTime, Duration, Utc};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use timer::Timer;
use uuid::Uuid;

use crate::json_response;
use crate::util::crypto::verify_helper;

use super::user::get_user;

const EXPIRE_TIME: u8 = 1;
const API_LIMIT: u64 = 1000;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Session {
    pub username: String,
    pub email: String,
    pub token: String,
    pub login_time: String,
    pub api_count: u64
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthInfo {
    pub username: String,
    pub password: String,
}

#[derive(Debug)]
pub enum AuthError {
    WrongPassword,
    NotLogin,
    TooFrequent,
    Expired,
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AuthError::WrongPassword => write!(f, "wrong password"),
            AuthError::NotLogin => write!(f, "not login"),
            AuthError::Expired => write!(f, "expired"),
            AuthError::TooFrequent => write!(f, "too frequent")
        }
    }
}

impl Error for AuthError {}

lazy_static! {
    pub static ref SESSION_POOL: Mutex < HashMap < String, Session > > = Mutex::new(HashMap::new());
    pub static ref API_CLEANER: Mutex<Timer> = Mutex::new(Timer::with_capacity(1));
    pub static ref API_COUNTER: Mutex<HashMap<String, u64>> = Mutex::new(HashMap::new());
}

pub async fn get_session(auth: BearerAuth) -> Result<Session, Box<dyn Error>> {
    let mut session_pool = SESSION_POOL.lock()?;
    match session_pool.get(auth.token()) {
        Some(session) => {
            if Utc::now().signed_duration_since(DateTime::parse_from_rfc2822(&session.login_time)?).num_days() < EXPIRE_TIME as i64 {
                let mut api_counter = API_COUNTER.lock()?;
                let count = api_counter.get_mut(&session.email).ok_or("unexpected error when try to get api counter")?;
                if *count > API_LIMIT {
                    Err(Box::new(AuthError::TooFrequent))
                } else {
                    *count += 1;
                    Ok::<Session, Box<dyn Error>>(session.clone())
                }
            } else {
                session_pool.remove(auth.token());
                Err(Box::new(AuthError::Expired))
            }
        }
        None => Err::<Session, Box<dyn Error>>(Box::new(AuthError::NotLogin)),
    }
}

pub async fn post_session(auth: AuthInfo) -> Result<Session, Box<dyn Error>> {
    let mut session_pool = SESSION_POOL.lock()?;
    let mut api_counter = API_COUNTER.lock()?;
    let user = get_user(None, &auth.username).await?;
    if verify_helper(&user.permanent_token, &auth.password) {
        if !api_counter.contains_key(&user.email) {
            api_counter.insert(user.email.clone(), 0);
        }
        let &count = api_counter.get(&user.email).unwrap();
        let uuid = Uuid::new_v4().to_string();
        let session = Session {
            email: user.email,
            username: user.username.clone(),
            token: uuid.clone(),
            login_time: Utc::now().to_rfc2822(),
            api_count: count
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
    API_CLEANER.lock().unwrap().schedule_repeating(Duration::hours(1), || {
        let mut pool = API_COUNTER.lock().unwrap();
        pool.iter_mut().for_each(|entry| { *entry.1 = 0 as u64 })
    });
    cfg.service(
        web::resource("/session")
            .route(web::post().to(post_session_handler))
            .route(web::get().to(get_session_handler))
            .route(web::delete().to(delete_session_handler))
    );
}