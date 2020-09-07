use std::error::Error;

use actix_web::{Responder, web};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use bcrypt::hash;
use mongodb::bson::{Bson, doc, from_bson};
use serde::{Deserialize, Serialize};
use mongodb::bson::Document;

use crate::{error, util::database::Database};
use crate::json_response;
use crate::resources::register_link::{validate_code, validate_email};
use crate::resources::session::{AuthInfo, get_session, post_session, Session};
use crate::util::crypto::BCRYPT_COST;
use crate::util::database::DEFAULT_DATABASE;
use crate::util::ops::PatchOperator;

#[derive(Debug, Deserialize, Serialize)]
pub struct User {
    pub(crate) username: String,
    pub(crate) email: String,
    pub(crate) permanent_token: String,
    pub(crate) learnt_course: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RegisterInfo {
    pub username: String,
    pub password: String,
    pub email: String,
    pub vcode: String,
}

pub async fn get_user(db: Option<&Database>, username: &str) -> Result<User, Box<dyn Error>> {
    let db = db.unwrap_or(&DEFAULT_DATABASE);
    let user_doc = db
        .cli.database(&db.name)
        .collection("User")
        .find_one(doc! {"username": username}, None)
        .await?
        .ok_or("user not found")?;
    Ok(from_bson::<User>(Bson::Document(user_doc))?)
}

pub async fn post_user(db: Option<&Database>, register_info: RegisterInfo) -> Result<Session, Box<dyn Error>> {
    let username = register_info.username;
    let password = register_info.password;
    let code = register_info.vcode;
    let email = register_info.email;
    let db = db.unwrap_or(&DEFAULT_DATABASE);
    validate_code(&email, &code)?;
    validate_email(&email)?;
    let hash = hash(&password, BCRYPT_COST)?;
    db.cli.database(&db.name)
        .collection("User")
        .insert_one(doc! {
            "username": username.to_string(),
            "permanent_token": hash,
            "email": email
        }, None).await?;
    Ok(post_session(AuthInfo {
        username: username.to_string(),
        password: password.to_string(),
    }).await?)
}

pub async fn patch_user(db: Option<&Database>, filter: Document, op: PatchOperator) -> Result<Bson, Box<dyn Error>> {
    let db = db.unwrap_or(&*DEFAULT_DATABASE);
    Ok(db
        .cli
        .database(&db.name)
        .collection("User")
        .update_one(filter, op.as_op(), None)
        .await?
        .upserted_id
        .ok_or("comment patch failed")?
    )
}

pub async fn patch_comment_handler(auth: BearerAuth, op: web::Json<PatchOperator>) -> impl Responder {
    let session = json_response!(get_session(auth).await).data.unwrap();
    let filter = doc! {"username": session.username};
    web::Json(json_response!(patch_user(None, filter, op.0).await))
}

async fn post_user_handler(req: web::Json<RegisterInfo>) -> impl Responder {
    web::Json(json_response!(post_user(None, req.0).await))
}

async fn get_user_handler(auth: BearerAuth) -> impl Responder {
    let session = json_response!(get_session(auth).await).data.unwrap();
    web::Json(json_response!(get_user(None, &session.username).await))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/user")
            .route(web::post().to(post_user_handler))
            .route(web::get().to(get_user_handler))
    );
}

#[cfg(test)]
mod test {
    use futures_await_test::async_test;
    use mongodb::bson::doc;
    use rand::Rng;
    use uuid::Uuid;

    use crate::resources::register_link::get_register_link;
    use crate::resources::user::{post_user, RegisterInfo};
    use crate::util::database::DEFAULT_DATABASE;

    #[async_test]
    async fn test_post_user() {
        let username = Uuid::new_v4().to_string();
        let mut rng = rand::thread_rng();
        let email = (rng.gen_range(1000_0000, 9999_9999) as u32).to_string();
        let code = get_register_link(&(email.clone() + "@sustech.edu.cn")).await.unwrap().code;

        assert!(post_user(None, RegisterInfo { username: username.clone(), password: "test".to_string(), vcode: code, email }).await.is_ok());
        let db = &DEFAULT_DATABASE;
        assert!(db.cli.database(&db.name).collection("User")
            .delete_one(doc! {"username": username}, None).await.is_ok());
    }
}