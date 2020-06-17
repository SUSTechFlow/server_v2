use actix_web::web;
use mongodb::bson::Bson;
use server_v2::resources::*;
use futures::executor::block_on;
use hex::ToHex;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    use actix_web::{App, HttpServer};
    let user = block_on(user::get_user(None, "11712009@mail.sustech.edu.cn"));
    let hash = String::from_utf8(base64::decode(&user.unwrap().permanent_token.replace("\n","")).unwrap());
    let res = bcrypt::verify("12345678", hash.unwrap().as_ref());
    println!("{:?}", res);
    HttpServer::new(|| {
        App::new()
            .configure(course::config)
            .configure(rate::config)
    })
        .bind("127.0.0.1:8088")?
        .run()
        .await
}