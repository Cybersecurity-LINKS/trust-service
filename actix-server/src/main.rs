// SPDX-FileCopyrightText: 2023 Fondazione LINKS
//
// SPDX-License-Identifier: APACHE-2.0

use std::{env, sync::{Mutex, RwLock, Arc}, path::PathBuf, fmt::format};
use actix_web::{web, App, HttpServer, middleware::Logger};
use iota_client::{secret::{SecretManager, stronghold::{StrongholdSecretManager, self}}, Client};
use iota_wallet::account_manager::AccountManager;
use trust_server::{utils::{setup_secret_manager, setup_account_manager}, storage::StorageType};
use trust_server::{controllers::{did_controller, proof_controller}, AppIotaState};
use trust_server::keycloak;
use mongodb::Client as MongoClient;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Type of the storage layer
    #[arg(short, long, required = true)]
    storage: String,
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let address = env::var("ADDR").expect("$ADDR must be set.");
    let port = env::var("PORT").expect("$PORT must be set.").parse::<u16>().unwrap();

    let usr = env::var("MONGO_INITDB_ROOT_USERNAME").expect("$MONGO_INITDB_ROOT_USERNAME must be set.");
    let pass = env::var("MONGO_INITDB_ROOT_PASSWORD").expect("$MONGO_INITDB_ROOT_PASSWORD must be set.");
    log::info!("Starting up on {}:{}", address, port);

    let secret_manager = setup_secret_manager(
        &env::var("STRONGHOLD_PASSWORD").unwrap(),
        &env::var("STRONGHOLD_PATH").unwrap(),
        &env::var("NON_SECURE_MNEMONIC").unwrap()
    ).await?;
    // TODO: request funds at the start if balance is low 

    let account_manager = Arc::new(RwLock::new(setup_account_manager(secret_manager).await?));
    let args = Args::parse();

    let storage: StorageType = match args.storage.as_str() {
        "mongo" => {
            log::info!("StorageType selected: MongoDB");
            let mongo_uri = env::var("MONGODB_URI").unwrap_or_else(|_| format!("mongodb://{}:{}@localhost:27017", usr, pass));
            let mongo_client = MongoClient::with_uri_str(mongo_uri).await.expect("failed to connect");
            //TODO: create an init function if the collections don't exist
            StorageType::MongoDB(mongo_client)
        }
        "keycloak" => {
            log::info!("StorageType selected: Keycloak");
            let http_client = reqwest::Client::new();

            // Get the admin token that provides access to the admin API.
            let admin_token = keycloak::get_admin_token(&http_client).await.unwrap();

            // Create a new realm.
            // A realm in Keycloak represents a security and administrative
            // domain where users, roles, and clients are managed.
            let realm_name = keycloak::create_realm(&http_client, &admin_token).await.unwrap();
            log::info!("Realm created: {}", realm_name);
            
            // Create a new client.
            // A client represents an application or service that wants to authenticate
            // and interact with Keycloak for user authentication and authorization.
            let client_data = keycloak::create_client(&http_client, &realm_name, &admin_token).await.unwrap();
            log::info!("Client created:\n{:#}", client_data);

            StorageType::Keycloak(
                keycloak::KeycloakSession{
                    admin_token: admin_token,
                    realm_name: realm_name,
                    client_data: client_data
                }
            )
        }
        
        _ => todo!()   
    };
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(
                AppIotaState {
                    account_manager: account_manager.clone()       
                })
            )
            .app_data(web::Data::new(storage.clone()))
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