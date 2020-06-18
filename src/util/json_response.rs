use std::error::Error;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonResponse<T>{
    pub(crate) data: Option<T>,
    pub(crate) error: Option<String>,
    pub(crate) meta: Option<String>,
}

