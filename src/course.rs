use serde::{Deserialize, Serialize};
use actix_web::web;

#[derive(Debug, Deserialize, Serialize)]
struct Course {
    cid: String,
    name: String,
    taught_by: Vec<String>,
    faculty: String,
}

async fn get_course()

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/course")
            .route("")
    );
}


