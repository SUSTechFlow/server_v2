use std::error::Error;

use futures::future;
use futures::stream::StreamExt;
use mongodb::bson::{Bson, doc, Document, from_bson};
use serde::{Deserialize, Serialize};
use crate::database::Database;

#[derive(Debug, Deserialize, Serialize)]
pub struct Course {
    cid: String,
    name: String,
    taught_by: Vec<String>,
    faculty: String,
}

pub async fn get_course(db: Option<&Database>, filter: Option<Document>) -> Result<Vec<Course>, Box<dyn Error>> {
    use crate::database::DEFAULT_DATABASE;
    let db = db.unwrap_or(&*DEFAULT_DATABASE);
    Ok(db
        .connect()
        .await?
        .database(&db.name)
        .collection("Course")
        .find(filter, None)
        .await?
        .map(|d| {
            Ok::<Bson, mongodb::error::Error>(Bson::Document(d?))
        })
        .filter(|x| future::ready(Result::is_ok(x)))
        .map(|d| {
            from_bson::<Course>(d.unwrap())
        })
        .filter(|x| future::ready(Result::is_ok(x)))
        .map(|x| x.unwrap())
        .collect::<Vec<Course>>()
        .await
    )
}

// pub fn config(cfg: &mut web::ServiceConfig) {
//     cfg.service(
//         web::scope("/course")
//             .route("")
//     );
// }

use futures_await_test::async_test;
#[async_test]
async fn test_get_course_10_times() {
    for _ in 0..10 {
        get_course(None, None).await.unwrap();
    }
}


