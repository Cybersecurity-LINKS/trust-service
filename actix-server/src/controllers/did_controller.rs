// SPDX-FileCopyrightText: 2023 Fondazione LINKS
//
// SPDX-License-Identifier: APACHE-2.0

use actix_web::get;
use actix_web::{web, HttpResponse, Responder, post};
use mongodb::Client as MongoClient;

use crate::{AppIotaState, storage};
use crate::services::did_service::create_did as create_did_service;
use crate::services::did_service::get_did_doc as get_did_doc_service;

use crate::DB_NAME;
use crate::storage::StorageType;

#[post("")] 
async fn create_did(app_iota_state: web::Data<AppIotaState>, storage: web::Data<StorageType>) -> impl Responder {
    let mut account_manager = app_iota_state.account_manager.write().unwrap();

    let resp = match create_did_service(&mut account_manager, storage.as_ref()).await {
        Ok(did) => {
            HttpResponse::Ok().body(did)
        },
        Err(_) => HttpResponse::InternalServerError().finish()
    };
    resp
}

#[get("/{did}")]
async fn get_did_doc(path: web::Path<String>) -> impl Responder {
    let resp = match get_did_doc_service(path.into_inner()).await {
        Ok(did_doc) => {
            HttpResponse::Ok().body(did_doc)
        },
        Err(_) => HttpResponse::InternalServerError().finish()
    };
    resp
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