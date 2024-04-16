// SPDX-FileCopyrightText: 2023 Fondazione LINKS
//
// SPDX-License-Identifier: APACHE-2.0

use actix_web::get;
use actix_web::{web, HttpResponse, post};

use crate::services::iota_state::IotaState;
use crate::services::mongodb_repo::MongoRepo;
use crate::errors::TrustServiceError;
use crate::models::user::User;

#[post("")] 
async fn create_did(
    iota_state: web::Data<IotaState>, 
    mongodb_repo: web::Data<MongoRepo>
) -> Result<HttpResponse, TrustServiceError> {
    log::info!("controller: create_did");

    let (iota_document, fragment) = iota_state.create_did().await?;
    log::info!("{:#}", iota_document);
    
    let user = User { did: iota_document.id().to_string(), fragment: fragment, assets: vec![] };
    mongodb_repo.store_user(user).await?;
    
    Ok(HttpResponse::Ok().body(iota_document.id().to_string()))
}

#[get("/{did}")]
async fn get_did_doc(
    path: web::Path<String>,
    iota_state: web::Data<IotaState>, 
) -> Result<HttpResponse, TrustServiceError> {
    log::info!("controller: get_did_doc");

    let did = path.into_inner();    
    let iota_document = iota_state.resolve_did(did.as_str()).await?; 
    Ok(HttpResponse::Ok().body(iota_document.to_string()))
}

// this function could be located in a different module
pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        // prefixes all resources and routes attached to it...
        web::scope("/dids")
        .service(create_did)
        .service(get_did_doc)            
    );
}