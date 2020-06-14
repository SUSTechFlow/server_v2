#[macro_export]
macro_rules! error {
    ($x:expr) => {
        match $x {
            Ok(res) => Ok(res),
            Err(err) => Err(Box::try_from(err).unwrap()),
        }
    }
}