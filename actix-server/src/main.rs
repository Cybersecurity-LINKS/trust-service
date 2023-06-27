use std::{env, sync::Mutex};
use actix_web::{web, App, HttpServer, middleware::Logger};
use iota_client::{secret::{SecretManager, stronghold::StrongholdSecretManager}, Client};
use trust_server::{controllers::{did_controller, proof_controller}, AppIotaState, utils::random_stronghold_path};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let address = env::var("ADDR").expect("$ADDR must be set.");
    let port = env::var("PORT").expect("$PORT must be set.").parse::<u16>().unwrap();

    log::info!("Starting up on {}:{}", address, port);
    
    HttpServer::new(|| {
        App::new()
            .app_data(web::Data::new(
                AppIotaState {
                    app_name: String::from("Actix Web"),
                    secret_manager: Mutex::new(SecretManager::Stronghold(
                        StrongholdSecretManager::builder()
                        .password("secure_password_2")
                        .build(random_stronghold_path()).unwrap(),
                    )),
                    client: Client::builder().with_primary_node(&env::var("NODE_URL").unwrap(), None).unwrap().finish().unwrap()
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
    .await
}