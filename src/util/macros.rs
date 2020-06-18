#[macro_export]
macro_rules! error {
    ($x:expr) => {
        match $x {
            Ok(res) => Ok(res),
            Err(err) => {
                use std::convert::TryFrom;
                Err(Box::try_from(err).unwrap())
            },
        }
    }
}

#[macro_export]
macro_rules! json_response {
    ($x:expr) => {
            match $x {
                Ok(v) => {
                    use crate::util::json_response::JsonResponse;
                    JsonResponse{data: Some(v), error: None, meta: None}
                },
                Err(e) => {
                    use crate::util::json_response::JsonResponse;
                    return web::Json(JsonResponse{data: None, error: Some(e.to_string()), meta:None});
                },
            }
    }
}