// SPDX-FileCopyrightText: 2024 Fondazione LINKS
//
// SPDX-License-Identifier: APACHE-2.0

use std::sync::Arc;

use actix_web::get;
use actix_web::{web, HttpResponse, post};
use ethers::abi::RawLog;
use ethers::contract::EthEvent;
use ethers::middleware::SignerMiddleware;
use ethers::providers::Provider;
use ethers::signers::Wallet;
use ethers::types::Address;

use crate::contracts::asset::Asset;
use crate::contracts::assetfactory::{AssetData, AssetFactory, NftMintedFilter};
use crate::controllers::AssetQuery;
use crate::dtos::{NftRequest, NftResponse};
use crate::services::mongodb_repo::MongoRepo;
use crate::errors::TrustServiceError;

#[post("/nfts")] 
async fn mint_nft(
    req: web::Json<NftRequest>, 
    mongodb_repo: web::Data<MongoRepo>,
    signer_data: web::Data<Arc<SignerMiddleware<Provider<ethers::providers::Http>, Wallet<ethers::core::k256::ecdsa::SigningKey>>>>,
) -> Result<HttpResponse, TrustServiceError> {
    log::info!("controller: mint_nft");
    // TODO: check if the did/user is the owner of the asset (assetId)
    let asset = mongodb_repo.get_asset(req.asset_id.clone()).await?;

    let address: Address = std::env::var("ASSET_FACTORY_ADDR").expect("$ASSET_FACTORY_ADDR must be set.").parse().map_err(|_| TrustServiceError::ContractAddressRecoveryError)?;
    let signer = signer_data.get_ref().clone();
    let asset_factory_sc = AssetFactory::new(address, signer);
    let asset_data = AssetData { 
        name: req.nft_alias.clone(), 
        symbol: req.nft_symbol.clone(),
        proof_id: asset.proof_id.clone(), 
        did: req.did.clone(), 
        asset_id: req.asset_id.clone(),
        license: req.license.clone()
    };
    let call = asset_factory_sc.tokenize(asset_data);
    let pending_tx = call.send().await.map_err(|err| TrustServiceError::ContractError(err.to_string()))?;

    // awaiting on the pending transaction resolves to a transaction receipt
    let receipt = pending_tx.confirmations(1).await.map_err(|err| TrustServiceError::ContractError(err.to_string()))?;
    let logs = receipt.ok_or(TrustServiceError::CustomError("No receipt".to_owned()))?.logs;

    // reading the log   
    for log in logs.iter() {
        let raw_log = RawLog {
            topics: log.topics.clone(),
            data: log.data.to_vec(),
        };
        // finding the event
        if let Ok(event) =  <NftMintedFilter as EthEvent>::decode_log(&raw_log){
            log::info!("Nft address: {:#x}", event.istance_address);
            // storing the address
            mongodb_repo.store_nft_addr(req.asset_id.clone(), format!("{:#x}", event.istance_address)).await?;
            return Ok(HttpResponse::Ok().finish());
        }
    }
    Err(TrustServiceError::CustomError("no NftMinted event found in the receipt".to_owned()))
}

#[get("/nfts")]
async fn get_nft_by_asset(
    query: web::Query<AssetQuery>, 
    mongodb_repo: web::Data<MongoRepo>,
    signer_data: web::Data<Arc<SignerMiddleware<Provider<ethers::providers::Http>, Wallet<ethers::core::k256::ecdsa::SigningKey>>>>,
) -> Result<HttpResponse, TrustServiceError> {
    log::info!("controller: read_nft");

    let asset = mongodb_repo.get_asset(query.asset_id.clone()).await?;
    let signer = signer_data.get_ref().clone();
    let nft_addr = asset.nft_addr.ok_or(TrustServiceError::MissingNftAddress)?;
    let addr: Address =  nft_addr.as_str().parse().map_err(|_| TrustServiceError::ContractAddressRecoveryError)?;
    log::info!("Nft address: {:#x}", addr);
    let asset_sc = Asset::new(addr, signer);

    let license = asset_sc.get_license().await.map_err(|err| TrustServiceError::ContractError(err.to_string()))?;
    let did = asset_sc.get_did().await.map_err(|err| TrustServiceError::ContractError(err.to_string()))?;
    //TODO: add check query.asset_id == asset_sc.get_asset_id()

    let respose = NftResponse{ asset_id: query.asset_id.clone(), nft_address: nft_addr, license, did };
    Ok(HttpResponse::Ok().json(respose))
}

pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg
    .service(mint_nft)
    .service(get_nft_by_asset);
}