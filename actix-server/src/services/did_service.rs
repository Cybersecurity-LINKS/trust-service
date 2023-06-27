use std::env;
use anyhow::Result;
use identity_iota::prelude::{IotaDocument, KeyPair};
use iota_client::block::address::Address;
use iota_client::secret::SecretManager;
use crate::utils::create_did as create_did_identity;
use iota_client::Client;

pub async fn create_did(secret_manager: &mut SecretManager) -> Result<()>  {

    let client = Client::builder().with_primary_node(&env::var("NODE_URL").unwrap(), None).unwrap().finish().unwrap();
    log::info!("Creating DID...");
    let (_, iota_document, key_pair_connector): (Address, IotaDocument, KeyPair) =
    match create_did_identity(&client, secret_manager).await {
        Ok(result) => result,
        Err(error) => return Err(error)
    };
    log::info!("{:#}", iota_document);
    Ok(())
}
