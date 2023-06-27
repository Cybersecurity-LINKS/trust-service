use actix_web::{web, HttpResponse, Responder, post};
use identity_iota::prelude::{IotaDocument, KeyPair};
use iota_client::block::address::Address;
use crate::AppIotaState;
use crate::utils::create_did as create_did_identity;

#[post("")] 
async fn create_did(data: web::Data<AppIotaState>) -> impl Responder {
    let app_name = &data.app_name; // <- get app_name
    let mut secret_manager = data.secret_manager.lock().unwrap();
    log::info!("Creating DID...");
    let (_, iota_document, key_pair_connector): (Address, IotaDocument, KeyPair) =
    match create_did_identity(&data.client, &mut secret_manager).await {
        Ok(result) => result,
        Err(error) => return HttpResponse::InternalServerError().body(error.to_string()),
    };
    log::info!("{:#}", iota_document);

    HttpResponse::Ok().body(format!("TODO: create_did() - Hello {app_name}!"))
}


// this function could be located in a different module
pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
         // prefixes all resources and routes attached to it...
        web::scope("/did")
            .service(create_did)
            
    );
}