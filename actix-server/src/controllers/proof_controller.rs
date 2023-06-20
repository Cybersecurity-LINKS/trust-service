use actix_web::{web, HttpResponse, Responder, Result, get, post};
use serde::Deserialize;
use crate::models::proof_dto::ProofRequestDTO;


#[get("/{proof_id}")]
async fn get_proof(info: web::Path<u32>) -> impl Responder {
    HttpResponse::Ok().body(format!(
        "Proof: {}",
        info.into_inner()
    ))
}

#[post("")]
async fn create_proof(req_body: web::Json<ProofRequestDTO>) -> impl Responder {
    HttpResponse::Ok().body(req_body.metadata_hash.clone())
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