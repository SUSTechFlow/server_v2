use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct Course {
    cid: String,
    name: String,
    taught_by: Vec<String>,
    faculty: String,
}

