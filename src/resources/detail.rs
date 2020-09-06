use crate::util::database::Database;
use futures::stream::StreamExt;
use futures::future;
use actix_web::{web, Responder};
use mongodb::bson::{Bson, doc, Document, from_bson};
use serde::{Deserialize, Serialize};
use crate::util::database::DEFAULT_DATABASE;

#[derive(Debug, Deserialize, Serialize)]
pub struct Detail {
    cid: String,
    english_name: String,
    open_by: String,
    test_method: String,
    credit: f32,
    detail: String,
}

async fn get_detail(db: Option<&Database>, filter: Option<Document>) -> Result<Vec<Detail>, Box<dyn std::error::Error>> {
    let db = db.unwrap_or(&*DEFAULT_DATABASE);
    Ok(db
        .cli
        .database(&db.name)
        .collection("Detail")
        .find(filter, None)
        .await?
        .map(|d| {
            Ok::<Bson, mongodb::error::Error>(Bson::Document(d?))
        })
        .filter(|x| future::ready(Result::is_ok(x)))
        .map(|d| {
            from_bson::<Detail>(d.unwrap())
        })
        .filter(|x| future::ready(Result::is_ok(x)))
        .map(|x| x.unwrap())
        .collect::<Vec<Detail>>()
        .await)
}

async fn get_detail_handler(req: web::Query<Bson>) -> impl Responder {
    use crate::json_response;
    web::Json(json_response!(get_detail(None, req.as_document().cloned()).await))
}