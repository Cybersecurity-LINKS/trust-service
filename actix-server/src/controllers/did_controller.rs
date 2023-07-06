use actix_web::{web, HttpResponse, Responder, post};
use mongodb::Client as MongoClient;

use crate::AppIotaState;
use crate::services::did_service::create_did as create_did_service;
use crate::DB_NAME;

#[post("")] 
async fn create_did(app_iota_state: web::Data<AppIotaState>, mongo_client: web::Data<MongoClient>) -> impl Responder {
    let mut account_manager = app_iota_state.account_manager.write().unwrap();
    let db = mongo_client.database(DB_NAME); // .expect("could not connect to database appdb");

    let resp = match create_did_service(&mut account_manager, db).await {
        Ok(_) => {
            HttpResponse::Ok().body(format!("TODO: create_did()"))
        },
        Err(_) => HttpResponse::InternalServerError().finish()
    };
    resp
}


// this function could be located in a different module
pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
         // prefixes all resources and routes attached to it...
        web::scope("/did")
            .service(create_did)
            
    );
}