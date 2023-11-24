// SPDX-FileCopyrightText: 2023 Fondazione LINKS
//
// SPDX-License-Identifier: APACHE-2.0

use actix_web::{web, HttpResponse, get, post};
use identity_iota::iota::IotaDocument;
use serde::Deserialize;

use crate::services::iota_state::IotaState;
use crate::services::mongodb_repo::MongoRepo;
use crate::dtos::proof_dto::ProofRequestDTO;
use crate::errors::TrustServiceError;
use crate::models::trustproof::TrustProof;


#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Info {
    asset_id: String,
}

#[get("/{proof_id}")]
async fn get_proof(
    path: web::Path<String>,
    iota_state: web::Data<IotaState>,
) -> Result<HttpResponse, TrustServiceError> {
    // TODO: check if it is a proof in the db
    let proof_id = path.into_inner();
    let proof = iota_state.resolve_proof(proof_id).await?;
    
    let publisher_document: IotaDocument = iota_state.resolve_did(proof.did_publisher.as_str()).await?;
    proof.verify(&publisher_document)?;

    Ok(HttpResponse::Ok().json(proof))
}

// this handler gets called if the query deserializes into `Info` successfully
// otherwise a 400 Bad Request error response is returned
//TODO: when sending a request the url should be encoded
#[get("")]
async fn get_proof_by_asset(
    info: web::Query<Info>, 
    iota_state: web::Data<IotaState>, 
    mongo_repo: web::Data<MongoRepo>
) -> Result<HttpResponse, TrustServiceError> {
    log::info!("controller: get_proof_by_asset");

    let proof_id = mongo_repo.get_proof_id(info.asset_id.clone()).await?;
    let proof = iota_state.resolve_proof(proof_id).await?;
    
    let publisher_document: IotaDocument = iota_state.resolve_did(proof.did_publisher.as_str()).await?;
    proof.verify(&publisher_document)?;

    Ok(HttpResponse::Ok().json(proof))
}

#[post("")] 
async fn create_proof(
    proof_dto: web::Json<ProofRequestDTO>, 
    iota_state: web::Data<IotaState>, 
    mongo_repo: web::Data<MongoRepo>
) -> Result<HttpResponse, TrustServiceError> {
    let did = proof_dto.did.as_str();
    let user = mongo_repo.get_user(did).await?;
    // Resolve the published DID Document
    let user_doc = iota_state.resolve_did(did).await?;

    log::info!("Creating trust proof...");
    let proof = TrustProof::new(
        &iota_state.key_storage,
        &user.fragment, 
        &proof_dto.metadata_hash, 
        &proof_dto.asset_hash, 
        &user_doc, 
        did.to_string()
    ).await?;

    log::info!("\n{:#?}", proof);
    let proof_id = iota_state.publish_proof(proof).await?.to_string();

    mongo_repo.store_proof_relationship(did, proof_id.clone(), proof_dto.asset_hash.clone()).await?;
    Ok(HttpResponse::Ok().body(proof_id)) 
}


// this function could be located in a different module
pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
         // prefixes all resources and routes attached to it...
        web::scope("/proofs")
            .service(get_proof)
            .service(get_proof_by_asset)
            .service(create_proof)
            
    );
}