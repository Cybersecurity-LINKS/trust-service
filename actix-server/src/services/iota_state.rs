// SPDX-FileCopyrightText: 2023 Fondazione LINKS
// SPDX-License-Identifier: APACHE-2.0

use std::str::FromStr;

use anyhow::Result;

use crypto::keys::bip39::Mnemonic;
use identity_iota::core::ToJson;
use identity_iota::iota::IotaDID;
use identity_iota::iota::block::output::AliasOutput;
use identity_iota::iota::IotaClientExt;
use identity_iota::iota::IotaDocument;
use identity_iota::iota::IotaIdentityClientExt;
use identity_iota::iota::NetworkName;
use identity_iota::storage::JwkDocumentExt;
use identity_iota::storage::JwkMemStore;
use identity_iota::storage::Storage;
use identity_iota::verification::MethodScope;

use identity_iota::verification::jws::JwsAlgorithm;
use identity_stronghold::StrongholdStorage;
use iota_sdk::Wallet;
use iota_sdk::client::Password;
use iota_sdk::client::secret::stronghold::StrongholdSecretManager;
use iota_sdk::client::Client;
use iota_sdk::types::block::address::Bech32Address;
use iota_sdk::types::block::output::OutputId;
use purity::account::PurityAccountExt;
use purity::utils::create_or_recover_wallet;
use purity::utils::request_faucet_funds;
use purity::utils::sync_print_balance;

use crate::errors::TrustServiceError;
use crate::models::tangle_proof::TangleProof;


pub type MemStorage = Storage<StrongholdStorage, StrongholdStorage>;
pub const MAIN_ACCOUNT: &str = "main-account";
pub const PROOF_TAG: &str = "trust-service-proofs"; 

pub struct IotaState {
  client: Client,
  _stronghold_storage: StrongholdStorage,
  pub key_storage: MemStorage,
  wallet: Wallet,
  address: Bech32Address,
  faucet: String,
}

impl IotaState {

  pub async fn init() -> Result<Self> {

    log::info!("Creating or recovering key storage...");

    let stronghold_pass = std::env::var("KEY_STORAGE_STRONGHOLD_PASSWORD")
    .expect("$KEY_STORAGE_STRONGHOLD_PASSWORD must be set.");

    let stronghold_path = std::env::var("KEY_STORAGE_STRONGHOLD_SNAPSHOT_PATH")
    .expect("$KEY_STORAGE_STRONGHOLD_SNAPSHOT_PATH must be set.");

    let mnemonic_string = std::env::var("KEY_STORAGE_MNEMONIC")
    .expect("$KEY_STORAGE_MNEMONIC must be set.");

    let faucet = std::env::var("FAUCET_URL").expect("$NODE_URL must be set.");
    let node_url = std::env::var("NODE_URL").expect("$NODE_URL must be set.");
    
    // Setup Stronghold secret_manager
    let stronghold = StrongholdSecretManager::builder()
    .password(Password::from(stronghold_pass))
    .build(stronghold_path)?;

    // Only required the first time, can also be generated with `manager.generate_mnemonic()?`
    let mnemonic = Mnemonic::from(mnemonic_string);

    match stronghold.store_mnemonic(mnemonic).await {
      Ok(()) => log::info!("Stronghold mnemonic stored"),
      Err(iota_sdk::client::stronghold::Error::MnemonicAlreadyStored) => log::info!("Stronghold mnemonic already stored"),
      Err(error) => panic!("Error: {:?}", error)
    }

    // Create a `StrongholdStorage`.
    // `StrongholdStorage` creates internally a `SecretManager` that can be
    // referenced to avoid creating multiple instances around the same stronghold snapshot.
    let stronghold_storage = StrongholdStorage::new(stronghold);

    // Create storage for key-ids and JWKs.
    //
    // In this example, the same stronghold file that is used to store
    // key-ids as well as the JWKs.
    let key_storage = Storage::new(stronghold_storage.clone(), stronghold_storage.clone());

    let client = Client::builder()
    .with_node(&node_url)?
    .finish()
    .await
    .map_err(|e| TrustServiceError::from(e))?;

    

    let wallet = create_or_recover_wallet().await?;
    // TODO: test with a new mnemonic
    let account = wallet.get_or_create_account(MAIN_ACCOUNT).await?;
    // Sync account to make sure account is updated with outputs from previous transactions
    // Sync the account to get the outputs for the addresses
    // Change to `true` to print the full balance report
    sync_print_balance(&account, false).await?;

    let service_address = &account.addresses().await?[0];
    // let governor_address = &account.generate_ed25519_addresses(1, None).await?[0];
    println!("Recovered address: {}", service_address.address());
    request_faucet_funds(&client, service_address.address(), faucet.as_str()).await?;
    let _ = account.sync(None).await?;

    Ok(IotaState{ client, _stronghold_storage: stronghold_storage, key_storage, wallet, faucet, address: service_address.to_owned().into_bech32() })
  }

  /// Creates a DID Document and publishes it in a new Alias Output.
  ///
  /// Its functionality is equivalent to the "create DID" example
  /// and exists for convenient calling from the other examples.
  pub async fn create_did(
    &self,
    // _address: Address
  ) -> Result<(IotaDocument, String), TrustServiceError> {
    // TODO: remove this
    // let address: Address = get_address_with_funds(client, secret_manager, FAUCET_ENDPOINT)
    //   .await
    //   .context("failed to get address with funds")?;
    
    let (document, fragment): (IotaDocument, String) = Self::create_did_document( &self).await?;

    //TODO: here the governor address is always the same, i.e. the service
    let alias_output: AliasOutput = self.client.new_did_output(self.address.into_inner(), document, None).await?;

    let secret_manager = self.wallet.get_secret_manager().write().await;
    let document: IotaDocument = self.client.publish_did_output(
      &secret_manager,
      alias_output
    ).await?;

    Ok((document, fragment))
  }

  /// Creates an example DID document with the given `network_name`.
  ///
  /// Its functionality is equivalent to the "create DID" example
  /// and exists for convenient calling from the other examples.
  async fn create_did_document(
    &self
  ) -> Result<(IotaDocument, String), TrustServiceError> {
    let network_name: NetworkName = self.client.network_name().await?;
    let mut document: IotaDocument = IotaDocument::new(&network_name);

    let fragment: String = document
      .generate_method(
        &self.key_storage,
        JwkMemStore::ED25519_KEY_TYPE,
        JwsAlgorithm::EdDSA,
        None,
        MethodScope::VerificationMethod,
      )
      .await?;

    Ok((document, fragment))
  }

  pub async fn resolve_did(
    &self,
    did: &str
  ) -> Result<IotaDocument, TrustServiceError> {
    log::info!("Resolving did...");
    log::info!("{}/identity-resolver/{}", std::env::var("EXPLORER_URL").unwrap(), did);
    match self.client.resolve_did(&IotaDID::try_from(did)?).await {
        Ok(iota_document) => Ok(iota_document),
        Err(err) => {
          log::info!("Error {}", err);
          Err(TrustServiceError::ResolveError(err))
        },
    }
  }

  pub async fn resolve_proof(
    &self,
    proof_id: String
  ) -> Result<TangleProof, TrustServiceError> {
   
    let output_id = OutputId::from_str(proof_id.as_str())?;

    log::info!("Reading trust proof from the tangle...");    
    let output = self.client.get_output(&output_id).await?.into_output();
    let metadata = output.features().expect("NO Features").metadata().expect("NO METADATA");
        
    // Extract metadata from output
    let trust_proof: TangleProof = serde_json::from_slice(metadata.data())?;
    log::info!("\n{:#?}", trust_proof);
    log::info!("{}/output/{}", std::env::var("EXPLORER_URL").unwrap(), &output_id);
  
    Ok(trust_proof)
  }

  pub async fn publish_proof(
    &self,
    proof: TangleProof
  ) -> Result<OutputId, TrustServiceError> {
   
    log::info!("Publishing trust proof msg...");

    let account = self.wallet.get_account(MAIN_ACCOUNT).await?;
    request_faucet_funds(&self.client, &self.address, self.faucet.as_str()).await?;
    let _ = account.sync(None).await?;
    

    // TODO: just publish the jws
    match account.write_data( 
        &self.address, 
        PROOF_TAG, 
        proof.to_json()?.as_str().as_bytes().to_vec(), 
        None
    ).await {
        Ok(proof_id) => Ok(proof_id),
        Err(_) => Err(TrustServiceError::WriteProofError),
    }

  }

}
  