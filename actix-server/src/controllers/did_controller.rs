use actix_web::{web, HttpResponse, Responder, post};


#[post("")] 
async fn create_did() -> impl Responder {
    HttpResponse::Ok().body("TODO: create_did()")
}


// this function could be located in a different module
pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
         // prefixes all resources and routes attached to it...
        web::scope("/did")
            .service(create_did)
            
    );
}