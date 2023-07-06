use std::env;
use anyhow::Result;
use identity_iota::core::ToJson;
use identity_iota::prelude::{KeyPair, KeyType};
use iota_client::Client;
use iota_wallet::account_manager::AccountManager;
use mongodb::Database;
use mongodb::bson::doc;
use crate::dtos::proof_dto::ProofRequestDTO;
use crate::models::proof::Proof;
use crate::{USER_COLL_NAME, PROOF_COLL_NAME, PROOF_TAG};
use crate::models::user::User;
use proof::trustproof::TrustProof;
use purity::account::PurityAccountExt;


pub async fn create_proof(proof_dto: ProofRequestDTO, account_manager: &mut AccountManager, mongo_db: Database) -> Result<()>  {
    
    // let secret_manager = account_manager.get_secret_manager();
    // let client = Client::builder().with_primary_node(&env::var("NODE_URL").unwrap(), None).unwrap().finish().unwrap();



    log::info!("Getting User information from db..."); // TODO: this will become a keycloak request   
    let collection = mongo_db.collection::<User>(USER_COLL_NAME); 
    let did = proof_dto.did.as_str();
    let filter = doc! {"did": did};

    let user = collection.find_one(Some(filter.clone()), None).await.unwrap().unwrap();


    let key_pair = KeyPair::try_from_private_key_bytes(KeyType::Ed25519, hex::decode(user.private_key).unwrap().as_slice()).unwrap();

    log::info!("Creating trust proof...");
    let proof = TrustProof::new(
        &proof_dto.metadata_hash, 
        &proof_dto.asset_hash, 
        &key_pair, 
        did.to_string()
    );

    // Read tag
    log::info!("\n{:?}\n", proof);

    log::info!("Publishing trust proof msg...");
    let account = account_manager.get_account(did.to_string()).await?;
    let _ = account.sync(None).await?;
    let bech32_address = account.addresses().await?[0].address().to_bech32();

    let trust_proof_id = account.write_data( 
        bech32_address, 
        PROOF_TAG, 
        proof.to_json()?.as_str().as_bytes().to_vec(),
        None
    ).await?;

    log::info!("Storing proof-asset relationship...");
    // let collection = mongo_db.collection::<User>(USER_COLL_NAME); 
    
    let proof = Proof { proof_id: trust_proof_id.to_string(), asset_id: proof_dto.asset_hash.clone()};

    // let result = collection.insert_one(proof, None).await;

    let update = doc! {
        "$push": {
            "proofs": proof
        }
    };
    let _result = collection.update_one(filter, update, None).await?;


    // let _ = match result {
    //     Ok(result) => result,
    //     Err(error) => {
    //         log::info!("{}", error);
    //         return Err(error.into())
    //     }
    // };

    log::info!("...End");
    Ok(())
    // Err("Error".to_string())
}
