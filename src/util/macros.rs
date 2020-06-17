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
macro_rules! handler {
    ($x:expr) => {
            match $x {
                Ok(v) => {
                    use crate::util::json_response::JsonResponse::data;
                    web::Json(data(v))
                },
                Err(e) => {
                    use crate::util::json_response::JsonResponse::error;
                    web::Json(error(e.to_string()))
                },
            }
    }
}