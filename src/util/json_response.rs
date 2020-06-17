use std::error::Error;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum JsonResponse<T, E> {
    data(T),
    error(E),
}

