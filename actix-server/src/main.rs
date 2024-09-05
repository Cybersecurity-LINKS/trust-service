// SPDX-FileCopyrightText: 2023 Fondazione LINKS
//
// SPDX-License-Identifier: APACHE-2.0

use std::sync::Arc;

use actix_web::{web, App, HttpServer, middleware::Logger};
use ethers::{middleware::SignerMiddleware, providers::{Http, Provider}, signers::{LocalWallet, Signer}};
use trust_server::{controllers::{did_controller, nft_controller, proof_controller}, services::{iota_state::IotaState, mongodb_repo::MongoRepo}};
use trust_server::controllers::log_controller;

#[actix_web::main]
async fn main() -> anyhow::Result<()> {

    // Load env files
    dotenv::from_path(".env").expect(".env file not found");
    dotenv::from_path(".mongo.env").expect(".mongo.env file not found");

    env_logger::init();

    let address = std::env::var("ADDR").expect("$ADDR must be set.");
    let port = std::env::var("PORT").expect("$PORT must be set.").parse::<u16>()?;
    
    let db: MongoRepo = MongoRepo::init().await;
    let db_data: web::Data<MongoRepo> = web::Data::new(db);

    let iota_state: IotaState = IotaState::init().await?;
    let iota_state_data: web::Data<IotaState> = web::Data::new(iota_state);

    // Initialize provider
    let rpc_provider =  std::env::var("RPC_PROVIDER").expect("$RPC_PROVIDER must be set.");
    let chain_id = std::env::var("CHAIN_ID").expect("$CHAIN_ID must be set.");
    // Transactions will be signed with the private key below
    let local_wallet = std::env::var("L2_PRIVATE_KEY").expect("$L2_PRIVATE_KEY must be set")
    .parse::<LocalWallet>()?
    .with_chain_id(chain_id.parse::<u64>().expect("CHAIN_ID is not a number"));


    log::info!("Initializing custom provider");
    let provider = Provider::<Http>::try_from(rpc_provider)?;
    let signer = Arc::new(SignerMiddleware::new(provider, local_wallet));
    let signer_data = web::Data::new(signer);

    log::info!("Starting up on {}:{}", address, port);
    HttpServer::new(move || {
        App::new()
            .app_data(iota_state_data.clone())
            .app_data(db_data.clone())
            .app_data(signer_data.clone())
            .service(web::scope("/api")
                .configure(did_controller::scoped_config)
                .configure(proof_controller::scoped_config)
                .configure(nft_controller::scoped_config)
                .configure(log_controller::scoped_config)
            )
            .wrap(Logger::default())
    })
    .bind((address, port))?
    .run()
    .await.map_err(anyhow::Error::from)
}