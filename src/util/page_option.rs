use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct PageOption {
    skip: i64,
    limit: i64,
    sort: i64,
}