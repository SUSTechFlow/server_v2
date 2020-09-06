use mongodb::bson::{Bson, doc, Document};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub enum PatchOperator {
    Set(String, Bson),
    Inc(String, i64),
}

impl PatchOperator {
    pub fn as_op(&self) -> Document {
        match self {
            PatchOperator::Set(field, new_value) => doc! {
                "$set": { field: new_value }
            },
            PatchOperator::Inc(field, amount) => doc! {
                "$inc": { field: amount }
            },
        }
    }
}