use server_v2::resources::*;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    use actix_web::{App, HttpServer};
    HttpServer::new(|| {
        App::new()
            .configure(course::config)
            .configure(rate::config)
            .configure(session::config)
            .configure(comment::config)
            .configure(detail::config)
    })
        .bind("127.0.0.1:8088")?
        .run()
        .await
}