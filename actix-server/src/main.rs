// SPDX-FileCopyrightText: 2023 Fondazione LINKS
//
// SPDX-License-Identifier: APACHE-2.0

use std::{env, sync::{RwLock, Arc}};
use actix_web::{web, App, HttpServer, middleware::Logger};
use iota_sdk::client::Client;
use purity::utils::{create_or_recover_wallet, sync_print_balance, request_faucet_funds};
use trust_server::{controllers::{did_controller, proof_controller}, AppIotaState, utils::create_or_recover_key_storage};
use mongodb::Client as MongoClient;
use trust_server::MAIN_ACCOUNT;

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let address = env::var("ADDR").expect("$ADDR must be set.");
    let port = env::var("PORT").expect("$PORT must be set.").parse::<u16>().unwrap();

    let usr = env::var("MONGO_INITDB_ROOT_USERNAME").expect("$MONGO_INITDB_ROOT_USERNAME must be set.");
    let pass = env::var("MONGO_INITDB_ROOT_PASSWORD").expect("$MONGO_INITDB_ROOT_PASSWORD must be set.");
    log::info!("Starting up on {}:{}", address, port);

    let wallet = create_or_recover_wallet().await?;
    let key_storage = create_or_recover_key_storage().await?;

    // TODO: request funds at the start if balance is low - test with a new mnemonic
    let account = wallet.get_or_create_account(MAIN_ACCOUNT).await?;
    // Sync account to make sure account is updated with outputs from previous examples
    // Sync the account to get the outputs for the addresses
    // Change to `true` to print the full balance report
    sync_print_balance(&account, false).await?;
    let governor_address = &account.generate_ed25519_addresses(1, None).await?[0];
    println!("Generated address: {}", governor_address.address());
    let client = Client::builder().with_node(&env::var("NODE_URL").unwrap())?.finish().await?;
    request_faucet_funds(&client, governor_address.address(), &env::var("FAUCET_URL").unwrap()).await?;
    let _ = account.sync(None).await?;

    let mongo_uri = env::var("MONGODB_URI").unwrap_or_else(|_| format!("mongodb://{}:{}@localhost:27017", usr, pass));
    let mongo_client = MongoClient::with_uri_str(mongo_uri).await.expect("failed to connect");
    //TODO: create an init function if the collections don't exist
    
    let wallet_arc = Arc::new(RwLock::new( wallet ));
    let storage_arc = Arc::new(RwLock::new( key_storage )); 
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(
                AppIotaState {
                    wallet: wallet_arc.clone(),
                    key_storage: storage_arc.clone(),
                })
            )
            .app_data(web::Data::new(mongo_client.clone()))
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