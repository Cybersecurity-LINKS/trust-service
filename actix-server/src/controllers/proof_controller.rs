use actix_web::{web, HttpResponse, Responder, get, post};
use mongodb::Client as MongoClient;

use crate::dtos::proof_dto::ProofRequestDTO;
use crate::services::proof_service::create_proof as create_proof_service;
use crate::services::proof_service::get_proof as get_proof_service;

use crate::AppIotaState;
use crate::DB_NAME;


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
            .service(create_proof)
            
    );
}