use lazy_static::lazy_static;
use std::sync::Mutex;
use std::collections::HashMap;
use super::user::User;
use actix_web::{web, HttpResponse, Responder};
use serde::{Serialize, Deserialize};
use actix_web::body::Body;
lazy_static! {
    static ref SESSION_POOL: Mutex<HashMap<String, User>> = Mutex::new(HashMap::new());
}
#[derive(Debug, Serialize, Deserialize)]
struct AuthInfo {
    username: String,
    password: String,
}

// pub async fn post_session_handler(req: web::Json<AuthInfo>) -> impl Responder {
//     let session_pool = SESSION_POOL.lock();
//
// }
