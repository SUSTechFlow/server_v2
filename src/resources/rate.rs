use actix_web::{Responder, web};
use futures::future;
use futures::stream::StreamExt;
use mongodb::bson::{Bson, doc, Document, from_bson};
use serde::{Deserialize, Serialize};

use crate::util::database::Database;

#[derive(Debug, Deserialize, Serialize)]
pub struct Rate {
    cid: String,
    name: String,
    ratings: f32,
    likes: f32,
    useful: f32,
    easy: f32,
}

async fn get_rate(db: Option<&Database>, filter: Option<Document>) -> Result<Vec<Rate>, Box<dyn std::error::Error>> {
    use crate::util::database::DEFAULT_DATABASE;
    let db = db.unwrap_or(&*DEFAULT_DATABASE);
    Ok(db
        .cli
        .database(&db.name)
        .collection("Rate")
        .find(filter, None)
        .await?
        .map(|d| {
            Ok::<Bson, mongodb::error::Error>(Bson::Document(d?))
        })
        .filter(|x| future::ready(Result::is_ok(x)))
        .map(|d| {
            from_bson::<Rate>(d.unwrap())
        })
        .filter(|x| future::ready(Result::is_ok(x)))
        .map(|x| x.unwrap())
        .collect::<Vec<Rate>>()
        .await)
}

async fn get_rate_handler(req: web::Query<Bson>) -> impl Responder {
    use crate::json_response;
    web::Json(json_response!(get_rate(None, req.as_document().cloned()).await))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/rate")
            .route(web::get().to(get_rate_handler))
    );
}