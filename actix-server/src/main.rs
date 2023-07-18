use std::{env, sync::{Mutex, RwLock, Arc}, path::PathBuf, fmt::format};
use actix_web::{web, App, HttpServer, middleware::Logger};
use iota_client::{secret::{SecretManager, stronghold::{StrongholdSecretManager, self}}, Client};
use iota_wallet::account_manager::AccountManager;
use trust_server::utils::{setup_secret_manager, setup_account_manager};
use trust_server::{controllers::{did_controller, proof_controller}, AppIotaState};
use mongodb::Client as MongoClient;

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
    let mongo_uri = env::var("MONGODB_URI").unwrap_or_else(|_| format!("mongodb://{}:{}@localhost:27017", usr, pass));
    let mongo_client = MongoClient::with_uri_str(mongo_uri).await.expect("failed to connect");
    //TODO: create an init function if the collections don't exist
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(
                AppIotaState {
                    account_manager: account_manager.clone()       
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