use bcrypt::hash;
use futures_await_test::async_test;
use mongodb::bson::{Bson, doc, from_bson};
use rand::Rng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{error, util::database::Database};
use crate::resources::register_link::{get_register_link, validate_code, validate_email};
use crate::resources::session::{AuthInfo, post_session, Session};
use crate::util::crypto::BCRYPT_COST;
use crate::util::database::DEFAULT_DATABASE;

#[derive(Debug, Deserialize, Serialize)]
pub struct User {
    pub(crate) username: String,
    pub(crate) email: String,
    pub(crate) permanent_token: String,
}

pub async fn get_user(db: Option<&Database>, username_or_email: &str) -> Result<User, Box<dyn std::error::Error>> {
    let db = db.unwrap_or(&DEFAULT_DATABASE);
    let user_doc = db
        .cli
        .database(&db.name)
        .collection("User")
        .find_one(
            doc! {"$or" :
                [
                    {"username": &username_or_email},
                    {"email": &username_or_email},
                ]
            }, None,
        )
        .await?;
    let user_doc = Bson::Document(user_doc.ok_or("user not found")?);
    error!(from_bson::<User>(user_doc))
}

pub async fn post_user(db: Option<&Database>, username: &str, email: &str, password: &str, code: &str) -> Result<Session, Box<dyn std::error::Error>> {
    let db = db.unwrap_or(&DEFAULT_DATABASE);
    validate_code(email, code)?;
    validate_email(email)?;
    let hash = hash(password, BCRYPT_COST)?;
    db.cli.database(&db.name)
        .collection("User")
        .insert_one(doc! {
            "username": username,
            "permanent_token": hash,
            "email": email
        }, None).await?;
    Ok(post_session(AuthInfo {
        username: username.to_string(),
        password: password.to_string(),
    }).await?)
}

#[cfg(test)]
mod test {
    use futures_await_test::async_test;
    use uuid::Uuid;
    use rand::Rng;
    use crate::resources::register_link::get_register_link;
    use crate::resources::user::post_user;
    use crate::util::database::DEFAULT_DATABASE;
    use mongodb::bson::doc;

    #[async_test]
    async fn test_post_user() {
        let username = &Uuid::new_v4().to_string();
        let mut rng = rand::thread_rng();
        let email = (rng.gen_range(1000_0000, 9999_9999) as u32).to_string();
        let code = get_register_link(&(email.clone() + "@sustech.edu.cn")).await.unwrap().code;

        assert!(post_user(None, username, &(email + "@sustech.edu.cn"), "test", &code).await.is_ok());
        let db = &DEFAULT_DATABASE;
        assert!(db.cli.database(&db.name).collection("User")
            .delete_one(doc! {"username": username}, None).await.is_ok());
    }
}