use mongodb::bson::{Bson, doc, Document};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub enum PatchOperator {
    AddToSet(String, Bson),
    Set(String, Bson),
    Inc(String, i64),
}

impl PatchOperator {
    pub fn as_op(&self) -> Document {
        match self {
            PatchOperator::AddToSet(field, elem) => doc! {
                "$addToSet": {field: elem}
            },
            PatchOperator::Set(field, new_value) => doc! {
                "$set": { field: new_value }
            },
            PatchOperator::Inc(field, amount) => doc! {
                "$inc": { field: amount }
            },
        }
    }
}