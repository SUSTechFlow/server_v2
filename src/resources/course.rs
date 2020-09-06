use actix_web::{Responder, web};
use futures::future;
use futures::stream::StreamExt;
use futures_await_test::async_test;
use mongodb::bson::{Bson, doc, Document, from_bson};
use serde::{Deserialize, Serialize};

use crate::json_response;
use crate::util::database::Database;
use crate::util::database::DEFAULT_DATABASE;

#[derive(Debug, Deserialize, Serialize)]
pub struct Course {
    cid: String,
    name: String,
    taught_by: Vec<Vec<String>>,
    faculty: String,
}

async fn get_course(db: Option<&Database>, filter: Option<Document>) -> Result<Vec<Course>, Box<dyn std::error::Error>> {
    let db = db.unwrap_or(&*DEFAULT_DATABASE);
    let filter = doc! { "$match": filter.unwrap_or(doc!{}) };
    let aggregator = doc! {
                "$group" : {
                    "_id" : "$cid",
                    "cid" : {"$first": "$cid"},
                    "name" : {"$first": "$name"},
                    "faculty" : {"$first": "$faculty"},
                    "taught_by" : {"$addToSet": "$taught_by"},
                }
            };
    Ok(db
        .cli
        .database(&db.name)
        .collection("Course")
        .aggregate(
            vec![filter, aggregator],
            None,
        )
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

async fn get_course_handler(req: web::Query<Bson>) -> impl Responder {
    web::Json(json_response!(get_course(None, req.as_document().cloned()).await))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/course")
            .route(web::get().to(get_course_handler))
    );
}

#[async_test]
async fn test_get_course_10_times() {
    for _ in 0..10 {
        get_course(None, None).await.unwrap();
    }
}


