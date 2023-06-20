use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use trust_server::controllers::proof_controller;
use dotenv::dotenv;
use std::{env, fmt::format};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    dotenv().ok();

    let address = format!("{}:{}", env::var("ADDR").expect("$ADDR must be set."), env::var("PORT").expect("$PORT must be set.").parse::<u16>().unwrap());

    log::info!("Starting up on {}", address);
    
    HttpServer::new(|| {
        App::new().service(         
            web::scope("/api")
            .configure(proof_controller::scoped_config))
    })
    .bind(address)?
    .run()
    .await
    //&env::var("BIND_ADDR").expect("BIND_ADDR must be set.");, &env::var("BIND_PORT").unwrap()
    
}