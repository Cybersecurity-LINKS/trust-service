// SPDX-FileCopyrightText: 2023 Fondazione LINKS
//
// SPDX-License-Identifier: APACHE-2.0

use std::env;
use std::str::FromStr;
use aes_gcm::aead::generic_array::GenericArray;
use anyhow::Result;
use identity_iota::core::ToJson;
use identity_iota::crypto::PublicKey;
use identity_iota::prelude::{KeyPair, KeyType, IotaDocument, IotaIdentityClientExt, IotaDID};
use iota_client::Client;
use iota_client::block::output::OutputId;
use iota_wallet::account_manager::AccountManager;
use mongodb::Database;
use mongodb::bson::doc;
use mongodb::options::FindOneOptions;
use purity::utils::get_metadata;
use serde_json::Value;
use crate::dtos::proof_dto::ProofRequestDTO;
use crate::models::proof::Proof;
use crate::{USER_COLL_NAME, PROOF_TAG};
use crate::models::user::User;
use proof::trustproof::TrustProof;
use purity::account::PurityAccountExt;
use anyhow::anyhow;

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Nonce, Key // Or `Aes128Gcm`
};
use base64::{Engine as _, engine::general_purpose};

// use ciborium::into_writer;
// use ciborium::from_reader;


pub async fn create_proof(proof_dto: ProofRequestDTO, account_manager: &mut AccountManager, mongo_db: Database) -> Result<String>  {
    
    // let secret_manager = account_manager.get_secret_manager();
    // let client = Client::builder().with_primary_node(&env::var("NODE_URL").unwrap(), None).unwrap().finish().unwrap();

    log::info!("Getting User information from db..."); // TODO: this will become a keycloak request   
    let collection = mongo_db.collection::<User>(USER_COLL_NAME); 
    let did = proof_dto.did.as_str();
    let filter = doc! {"did": did};

    let user = collection.find_one(Some(filter.clone()), None).await.unwrap().unwrap();


    
    //TODO: understand if this should be in global state
    let aes_key_vec = general_purpose::STANDARD.decode(&env::var("ENC_KEY").expect("$ENC_KEY must be set."))?;
    let aes_key = Key::<Aes256Gcm>::from_slice(aes_key_vec.as_slice());
    let cipher = Aes256Gcm::new(aes_key);

    let sk_bytes = cipher.decrypt(&GenericArray::clone_from_slice( user.nonce.as_slice()), user.private_key.as_ref()).unwrap();
    log::info!("Dec sk: {:?}",sk_bytes);
    let key_pair = KeyPair::try_from_private_key_bytes(KeyType::Ed25519, &sk_bytes).unwrap();
    // let key_pair = KeyPair::try_from_private_key_bytes(KeyType::Ed25519, hex::decode(user.private_key).unwrap().as_slice()).unwrap();


    log::info!("Creating trust proof...");
    let proof = TrustProof::new(
        &proof_dto.metadata_hash, 
        &proof_dto.asset_hash, 
        &key_pair, 
        did.to_string()
    );

    // Read tag
    log::info!("\n{:#?}", proof);

    log::info!("Publishing trust proof msg...");
    let account = account_manager.get_account(did.to_string()).await?;
    let _ = account.sync(None).await?;
    let bech32_address = account.addresses().await?[0].address().to_bech32();

    // CBOR
    // let mut cbor_proof = Vec::new();
    // into_writer(&proof, &mut cbor_proof)?;
    // log::info!("CBOR:\n{:?}", cbor_proof);

    let trust_proof_id = account.write_data( 
        bech32_address, 
        PROOF_TAG, 
        // cbor_proof,
        proof.to_json()?.as_str().as_bytes().to_vec(), 
        None
    ).await?;

    log::info!("Storing proof-asset relationship...");
    let proof = Proof { proof_id: trust_proof_id.to_string(), asset_id: proof_dto.asset_hash.clone()};
    let update = doc! {
        "$push": {
            "proofs": proof
        }
    };
    let _result = collection.update_one(filter, update, None).await?;

    log::info!("...End");
    Ok(trust_proof_id.to_string())
}

//TODO: handle not found
pub async fn get_proof(proof_id: String) -> Result<String> {

    let client = Client::builder().with_primary_node(&env::var("NODE_URL")?, None)?.finish()?;
    // Take the output ID from command line argument or use a default one.
    let output_id = OutputId::from_str(&proof_id)?;

    log::info!("Reading trust proof from the tangle...");    
    let output_metadata = client.get_output(&output_id).await?;

    // CBOR
    // let trust_proof: TrustProof = from_reader(get_metadata(output_metadata.clone())?.as_slice())?;
    // log::info!("CBOR:\n{:?}",trust_proof);
    
    // Extract metadata from output
    let trust_proof: TrustProof = serde_json::from_slice(&get_metadata(output_metadata)?)?;
    log::info!("\n{:#?}", trust_proof);
    log::info!("{}/output/{}", std::env::var("EXPLORER_URL").unwrap(), &output_id);

    
    log::info!("Reading did document from the tangle...");
    log::info!("{}/identity-resolver/{}", std::env::var("EXPLORER_URL").unwrap(), &trust_proof.did_publisher);
    let publisher_document: IotaDocument = client.resolve_did(&IotaDID::from_str(trust_proof.did_publisher.as_str())?).await?;

    let publisher_public_key = PublicKey::from(
        publisher_document.core_document()
        .verification_method()
        .first()
        .unwrap()
        .data()
        .try_decode()
        .unwrap()
    );

    log::info!("Verifying proof...");
    match trust_proof.verify(&publisher_public_key) {
        Ok(_) =>  serde_json::to_string(&trust_proof).map_err(anyhow::Error::from),
        Err(e) => {
            log::info!("{}", e);
            //TODO: define custom error
            Err(anyhow!("Proof verification failed"))
        }
    }
}

pub async fn get_proof_by_asset(asset_id: String, mongo_db: Database) -> Result<String> {

    log::info!("Getting Asset information from db..."); // TODO: this will become a keycloak request   
    let collection = mongo_db.collection::<User>(USER_COLL_NAME); 
    let projected_collection = collection.clone_with_type::<Value>();
    log::info!("Searching for asset: {:#?}", asset_id);

    // Define the filter query
    let filter = doc! {
        "proofs": {
          "$elemMatch": {
            "assetId": asset_id.clone()
          }
        }
    };

    // Define the projection query
    // Use FindOptions::builder() to set the projection
    let find_options = FindOneOptions::builder().projection(doc! {
        "proofs.$": 1,
        "_id": 0
    }).build();

    let proof_id = match projected_collection.find_one(Some(filter), find_options).await {
        Ok(Some(user)) => {
            if let Some(proof_id) = user["proofs"][0]["proofId"].as_str() {
                log::info!("proofId: {}", proof_id);
                Ok(proof_id.to_owned())
            } else {
                Err(anyhow!("ProofId not found in the document."))
            }      
        },
        Ok(None) => Err(anyhow!("No asset found with id: {}", asset_id)),
        Err(err) => Err(anyhow!("Error: {}", err))
    };
    get_proof(proof_id?).await
}
