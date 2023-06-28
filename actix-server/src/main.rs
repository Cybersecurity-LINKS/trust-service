use std::{env, sync::{Mutex, RwLock, Arc}, path::PathBuf};
use actix_web::{web, App, HttpServer, middleware::Logger};
use iota_client::{secret::{SecretManager, stronghold::{StrongholdSecretManager, self}}, Client};
use iota_wallet::account_manager::AccountManager;
use trust_server::utils::{setup_secret_manager, setup_account_manager};
use trust_server::{controllers::{did_controller, proof_controller}, AppIotaState, utils::random_stronghold_path};

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let address = env::var("ADDR").expect("$ADDR must be set.");
    let port = env::var("PORT").expect("$PORT must be set.").parse::<u16>().unwrap();

    log::info!("Starting up on {}:{}", address, port);
    let stronghold_path = &env::var("STRONGHOLD_PATH").unwrap();

    let secret_manager = setup_secret_manager(
        &env::var("STRONGHOLD_PASSWORD").unwrap(),
        stronghold_path,
        &env::var("NON_SECURE_MNEMONIC").unwrap()
    ).await?;

    let manager = Arc::new(RwLock::new(setup_account_manager(secret_manager).await?));
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(
                AppIotaState {
                    account_manager: manager.clone()       
                })
            )
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