use actix_web::middleware::DefaultHeaders;
use actix_web::{middleware, web, App, HttpRequest, HttpResponse, HttpServer, Responder};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let scyllauser = arguments::parse(std::env::args())
        .unwrap()
        .get::<String>("scyllausername");

        let scyllapassword = arguments::parse(std::env::args())
        .unwrap()
        .get::<String>("scyllapassword");

    // Create a new HTTP server.
    let builder = HttpServer::new(move || {
        App::new()
            .wrap(
                DefaultHeaders::new()
                    .add(("Server", "Adora"))
            )
            .wrap(actix_block_ai_crawling::BlockAi)
    })
    .workers(4);

    // Bind the server to port 8080.
    let _ = builder.bind("127.0.0.1:5401").unwrap().run().await;

    Ok(())
}