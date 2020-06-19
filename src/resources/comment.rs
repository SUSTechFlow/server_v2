use serde::{Deserialize, Serialize};
use futures::stream::StreamExt;
use futures::future;
use std::error::Error;
use mongodb::bson::{Bson, doc, Document, from_bson};
use crate::util::database::Database;
use actix_web::{web, Responder};

#[derive(Debug, Deserialize, Serialize)]
enum Gpa {
    #[serde(rename(serialize = "A+", deserialize = "A+"))]
    APlus,
    A,
    #[serde(rename(serialize = "A-", deserialize = "A-"))]
    AMinus,
    #[serde(rename(serialize = "B+", deserialize = "B+"))]
    BPlus,
    B,
    #[serde(rename(serialize = "B-", deserialize = "B-"))]
    BMinus,
    #[serde(rename(serialize = "C+", deserialize = "C+"))]
    CPlus,
    C,
    #[serde(rename(serialize = "C-", deserialize = "C-"))]
    CMinus,
    #[serde(rename(serialize = "D+", deserialize = "D+"))]
    DPlus,
    D,
    #[serde(rename(serialize = "D-", deserialize = "D-"))]
    DMinus,
    F,
    P,
    X,
}

#[derive(Debug, Deserialize, Serialize)]
enum Term {
    #[serde(rename(serialize = "春", deserialize = "春"))]
    Spring,
    #[serde(rename(serialize = "夏", deserialize = "夏"))]
    Summer,
    #[serde(rename(serialize = "秋", deserialize = "秋"))]
    Fall,
    #[serde(rename(serialize = "冬", deserialize = "冬"))]
    Winter,
}

#[derive(Debug, Deserialize, Serialize)]
struct Rate {
    likes: f32,
    useful: f32,
    easy: f32,
    ratings: f32,
}

#[derive(Debug, Deserialize, Serialize)]
struct Comment {
    gpa: Option<Gpa>,
    cid: String,
    content: String,
    comment_by: Option<String>,
    term: Term,
    willing: bool,
    anonymous: bool,
    rate: Rate,
    taught: Vec<String>,
    helpful: Option<usize>,
    not_helpful: Option<usize>,
    year: usize,
    month: usize,
    day: usize,
}

async fn get_comment(db: Option<&Database>, filter: Option<Document>) -> Result<Vec<Comment>, Box<dyn Error>> {
    use crate::util::database::DEFAULT_DATABASE;
    let db = db.unwrap_or(&*DEFAULT_DATABASE);
    let filter = filter.unwrap_or(doc!{});
    Ok(db
        .connect()
        .await?
        .database(&db.name)
        .collection("Comment")
        .find(filter, None)
        .await?
        .map(|d| {
            Ok::<Bson, mongodb::error::Error>(Bson::Document(d?))
        })
        .filter(|x| future::ready(Result::is_ok(x)))
        .map(|d| {
            from_bson::<Comment>(d.unwrap())
        })
        .filter(|x| future::ready(Result::is_ok(x)))
        .map(|x| x.unwrap())
        .map(|x| {
            let mut x = x;
            if !x.willing {
                x.gpa = None }
            if x.anonymous {
                x.comment_by = None
            }
            x
        })
        .collect::<Vec<Comment>>()
        .await
    )
}

async fn get_comment_handler(req: web::Query<Bson>) -> impl Responder {
    use crate::json_response;
    web::Json(json_response!(get_comment(None, req.as_document().cloned()).await))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/comment")
            .route(web::get().to(get_comment_handler))
    );
}