use actix_web::{web, HttpResponse, Responder, post};

use crate::AppIotaState;
use crate::services::did_service::create_did as create_did_service;

#[post("")] 
async fn create_did(data: web::Data<AppIotaState>) -> impl Responder {
    let mut account_manager = data.account_manager.write().unwrap();

    let resp = match create_did_service(&mut account_manager).await {
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