use mongodb::bson::{Bson, doc, from_bson};
use serde::{Deserialize, Serialize};

use crate::{error, util::database::Database};

#[derive(Debug, Deserialize, Serialize)]
pub struct User {
    pub(crate) username: String,
    pub(crate) email: String,
    pub(crate) permanent_token: String,
}

pub(crate) async fn get_user(db: Option<&Database>, username_or_email: &str) -> Result<User, Box<dyn std::error::Error>> {
    use crate::util::database::DEFAULT_DATABASE;
    let db = db.unwrap_or(&*DEFAULT_DATABASE);
    let user_doc = db
        .connect()
        .await?
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




