// SPDX-FileCopyrightText: 2023 Fondazione LINKS
//
// SPDX-License-Identifier: APACHE-2.0

use actix_web::{web, App, HttpServer, middleware::Logger};
use trust_server::{controllers::{did_controller, proof_controller}, services::{mongodb_repo::MongoRepo, iota_state::IotaState}};

#[actix_web::main]
async fn main() -> anyhow::Result<()> {

    dotenv::from_path(".env").expect(".env file not found");
    dotenv::from_path(".mongo.env").expect(".mongo.env file not found");

    env_logger::init();

    let address = std::env::var("ADDR").expect("$ADDR must be set.");
    let port = std::env::var("PORT").expect("$PORT must be set.").parse::<u16>()?;
    
    let db: MongoRepo = MongoRepo::init().await;
    let db_data: web::Data<MongoRepo> = web::Data::new(db);

    let iota_state: IotaState = IotaState::init().await?;
    let iota_state_data: web::Data<IotaState> = web::Data::new(iota_state);

    log::info!("Starting up on {}:{}", address, port);
    HttpServer::new(move || {
        App::new()
            .app_data(iota_state_data.clone())
            .app_data(db_data.clone())
            .service(web::scope("/api")
                .configure(did_controller::scoped_config)
                .configure(proof_controller::scoped_config)
            )
            .wrap(Logger::default())
    })
    .bind((address, port))?
    .run()
    .await.map_err(anyhow::Error::from)
}