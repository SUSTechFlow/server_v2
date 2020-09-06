use std::error::Error;

use actix_web::{Responder, web};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use futures::future;
use futures::stream::StreamExt;
use mongodb::bson::{Bson, doc, Document, from_bson, to_bson};
use serde::{Deserialize, Serialize};

use crate::json_response;
use crate::resources::session::get_session;
use crate::util::database::Database;
use crate::util::database::DEFAULT_DATABASE;
use crate::util::ops::PatchOperator;

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
pub struct Comment {
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

pub async fn get_comment(db: Option<&Database>, filter: Option<Document>) -> Result<Vec<Comment>, Box<dyn Error>> {
    let db = db.unwrap_or(&*DEFAULT_DATABASE);
    let filter = filter.unwrap_or(doc! {});
    Ok(db
        .cli
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
                x.gpa = None
            }
            if x.anonymous {
                x.comment_by = None
            }
            x
        })
        .collect::<Vec<Comment>>()
        .await
    )
}

pub async fn post_comment(db: Option<&Database>, comment: &Comment) -> Result<Bson, Box<dyn Error>> {
    let db = db.unwrap_or(&*DEFAULT_DATABASE);
    let cid = &comment.cid;
    let comment_by = &comment.comment_by.as_ref().unwrap();
    let comment = to_bson(comment)?.as_document().ok_or("failed to transfer Bson to Document")?.clone();
    Ok(db
        .cli
        .database(&db.name)
        .collection("Comment")
        .replace_one(doc! {
            "cid": cid,
            "comment_by": comment_by
        }, comment, None)
        .await?
        .upserted_id
        .ok_or("comment upsert failed")?
    )
}

pub async fn patch_comment(db: Option<&Database>, filter: Document, op: PatchOperator) -> Result<Bson, Box<dyn Error>> {
    let db = db.unwrap_or(&*DEFAULT_DATABASE);

    Ok(db
        .cli
        .database(&db.name)
        .collection("Comment")
        .update_one(filter, op.as_op(), None)
        .await?
        .upserted_id
        .ok_or("comment patch failed")?
    )
}

pub async fn get_comment_handler(req: web::Query<Bson>) -> impl Responder {
    web::Json(json_response!(get_comment(None, req.as_document().cloned()).await))
}

pub async fn post_comment_handler(auth: BearerAuth, mut comment: web::Json<Comment>) -> impl Responder {
    let session = json_response!(get_session(auth).await).data.unwrap();
    comment.comment_by = Some(session.username);
    web::Json(json_response!(post_comment(None, &comment.into_inner()).await))
}

pub async fn patch_comment_handler(auth: BearerAuth, req: web::Query<Bson>, op: web::Json<PatchOperator>) -> impl Responder {
    let session = json_response!(get_session(auth).await).data.unwrap();
    let mut filter = req.as_document().cloned().unwrap_or(doc! {});
    filter.insert("comment_by", session.username);
    web::Json(json_response!(patch_comment(None, filter, op.0).await))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/comment")
            .route(web::get().to(get_comment_handler))
            .route(web::post().to(post_comment_handler))
            .route(web::patch().to(patch_comment_handler))
    );
}