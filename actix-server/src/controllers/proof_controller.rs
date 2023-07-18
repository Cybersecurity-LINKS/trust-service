use actix_web::{web, HttpResponse, Responder, get, post};
use mongodb::Client as MongoClient;
use serde::Deserialize;

use crate::dtos::proof_dto::ProofRequestDTO;
use crate::services::proof_service::create_proof as create_proof_service;
use crate::services::proof_service::get_proof as get_proof_service;
use crate::services::proof_service::get_proof_by_asset as get_proof_by_asset_service;


use crate::AppIotaState;
use crate::DB_NAME;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Info {
    asset_id: String,
}

#[get("/{proof_id}")]
async fn get_proof(path: web::Path<String>) -> impl Responder {

    let resp = match get_proof_service(path.into_inner()).await {
        Ok(proof) => {
            HttpResponse::Ok().body(proof)
        },
        Err(_) => HttpResponse::InternalServerError().finish()
    };
    resp
}

// this handler gets called if the query deserializes into `Info` successfully
// otherwise a 400 Bad Request error response is returned
//TODO: when sending a request the url should be encoded
#[get("")]
async fn get_proof_by_asset(info: web::Query<Info>, mongo_client: web::Data<MongoClient>) -> impl Responder {
    let db: mongodb::Database = mongo_client.database(DB_NAME); // .expect("could not connect to database appdb");
    let resp = match get_proof_by_asset_service(info.asset_id.clone(), db).await {
        Ok(proof) => {
            HttpResponse::Ok().body(proof)
        },
        Err(_) => HttpResponse::InternalServerError().finish()
    };
    resp
}

// TODO: add schema validation
#[post("")] 
async fn create_proof(req_body: web::Json<ProofRequestDTO>, app_iota_state: web::Data<AppIotaState>, mongo_client: web::Data<MongoClient>) -> impl Responder {
    let mut account_manager = app_iota_state.account_manager.write().unwrap();
    let db = mongo_client.database(DB_NAME); // .expect("could not connect to database appdb");
    let resp = match create_proof_service(req_body.into_inner(), &mut account_manager, db).await {
        Ok(proof_id) => {
            HttpResponse::Ok().body(proof_id)
        },
        Err(_) => HttpResponse::InternalServerError().finish()
    };
    resp
}


// this function could be located in a different module
pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
         // prefixes all resources and routes attached to it...
        web::scope("/trust-proof")
            .service(get_proof)
            .service(get_proof_by_asset)
            .service(create_proof)
            
    );
}