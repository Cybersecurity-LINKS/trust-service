use actix_web::{web, HttpResponse, Responder, get, post};
use crate::dtos::proof_dto::ProofRequestDTO;
use crate::services::proof_service::create_proof as create_proof_service;

#[get("/{proof_id}")]
async fn get_proof(path: web::Path<u32>) -> impl Responder {
    HttpResponse::Ok().body(format!(
        "Proof: {}",
        path.into_inner()
    ))

    // HttpResponse::BadRequest().body("Bad data");
}

// TODO: add schema validation
#[post("")] 
async fn create_proof(req_body: web::Json<ProofRequestDTO>) -> impl Responder {
    let resp = match  create_proof_service(req_body.into_inner()) {
        Ok(_) => HttpResponse::Ok().body("ciao"),
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