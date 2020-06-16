use actix_web::{HttpResponse, web};
use actix_web::body::Body;
use futures::future;
use futures::stream::StreamExt;
use mongodb::bson::{Bson, doc, Document, from_bson};
use serde::{Deserialize, Serialize};

use crate::database::Database;

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
    use crate::database::DEFAULT_DATABASE;
    let db = db.unwrap_or(&*DEFAULT_DATABASE);
    Ok(db
        .connect()
        .await?
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

async fn get_rate_handler(req: web::Query<Bson>) -> Result<HttpResponse<Body>, actix_web::Error> {
    let injson = match get_rate(None, req.as_document().cloned()).await {
        Ok(v) => serde_json::to_string(&v),
        Err(e) => serde_json::to_string(&e.to_string()),
    };
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(injson.unwrap()))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/rate")
            .route(web::get().to(get_rate_handler))
    );
}