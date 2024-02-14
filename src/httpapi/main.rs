use actix_web::middleware::DefaultHeaders;
use actix_web::{middleware, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use scylla::{Session, SessionBuilder};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let scyllauser = arguments::parse(std::env::args())
        .unwrap()
        .get::<String>("scyllausername")
        .unwrap();

    let scyllapassword = arguments::parse(std::env::args())
        .unwrap()
        .get::<String>("scyllapassword")
        .unwrap();

    let session: Session = SessionBuilder::new()
        .known_node("127.0.0.1:9042")
        .user(scyllauser, scyllapassword)
        .build()
        .await
        .unwrap();

    // Create a new HTTP server.
    let builder = HttpServer::new(move || {
        App::new()
            .wrap(DefaultHeaders::new().add(("Server", "Adora")))
            .wrap(actix_block_ai_crawling::BlockAi)
    })
    .workers(4);

    // Bind the server to port 8080.
    let _ = builder.bind("127.0.0.1:5401").unwrap().run().await;

    Ok(())
}
