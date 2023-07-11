use std::env;
use std::str::FromStr;
use anyhow::Result;
use identity_iota::core::ToJson;
use identity_iota::crypto::PublicKey;
use identity_iota::prelude::{KeyPair, KeyType, IotaDocument, IotaIdentityClientExt, IotaDID};
use iota_client::Client;
use iota_client::block::output::OutputId;
use iota_wallet::account_manager::AccountManager;
use mongodb::Database;
use mongodb::bson::doc;
use purity::utils::get_metadata;
use crate::dtos::proof_dto::ProofRequestDTO;
use crate::models::proof::Proof;
use crate::{USER_COLL_NAME, PROOF_COLL_NAME, PROOF_TAG};
use crate::models::user::User;
use proof::trustproof::TrustProof;
use purity::account::PurityAccountExt;
use anyhow::anyhow;

pub async fn create_proof(proof_dto: ProofRequestDTO, account_manager: &mut AccountManager, mongo_db: Database) -> Result<String>  {
    
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

pub async fn get_proof(proof_id: String) -> Result<String> {

    let client = Client::builder().with_primary_node(&env::var("NODE_URL")?, None)?.finish()?;
    // Take the output ID from command line argument or use a default one.
    let output_id = OutputId::from_str(&proof_id)?;

    log::info!("Reading trust proof from the tangle...");    
    let output_metadata = client.get_output(&output_id).await?;
    // Extract metadata from output
    let trust_proof: TrustProof = serde_json::from_slice(&get_metadata(output_metadata)?)?;
    log::info!("{:#?}", trust_proof);
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