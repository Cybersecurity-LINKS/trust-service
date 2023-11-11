// SPDX-FileCopyrightText: 2023 Fondazione LINKS
//
// SPDX-License-Identifier: APACHE-2.0

use anyhow::Result;
use crypto::hashes::Digest;
use crypto::hashes::blake2b::Blake2b256;
use base64::{Engine as _, engine::general_purpose};

use identity_iota::credential::Jws;
use identity_iota::document::verifiable::JwsVerificationOptions;
use identity_iota::prelude::IotaDocument;
use identity_iota::storage::JwkDocumentExt;
use identity_iota::storage::JwsSignatureOptions;
use identity_eddsa_verifier::EdDSAJwsVerifier;
use serde::Serialize;
use serde::Deserialize;

use crate::utils::MemStorage;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrustProof {
    metadata_digest: String,
    dataset_digest: String,
    jws: String,
    pub did_publisher: String, //TODO: beware of pub
}

// TODO: implement two new, one that compute the hash, one that take as input a whole message
impl TrustProof {
    
    pub async fn new(
        storage: &MemStorage,
        fragment: &String,
        metadata_digest: &String,
        dataset_digest: &String,
        iota_document: &IotaDocument,
        did_publisher: String
    ) -> Result<Self> {

        // TODO: (case 1 we receive the hash computed from another service) if the input are already a digest, is this necessary?
        let digest_metadata: [u8; 32] = Blake2b256::digest(metadata_digest.as_bytes()).as_slice().try_into().expect("Wrong length");
        let digest_dataset: [u8; 32]  = Blake2b256::digest(dataset_digest.as_bytes()).as_slice().try_into().expect("Wrong length");

        let digests_sum = [digest_metadata, digest_dataset].concat();

        // Compute signature
        let jws = iota_document.create_jws(&storage, &fragment, &digests_sum, &JwsSignatureOptions::default()).await?;
        // Verify signature
        let _decoded_jws = iota_document.verify_jws(
            &jws,
            None,
            &EdDSAJwsVerifier::default(),
            &JwsVerificationOptions::default(),
        )?; // TODO: catch the error on caller and log ("Signature NOT Valid")
        
        Ok(Self{
            metadata_digest: general_purpose::STANDARD.encode(digest_metadata), 
            dataset_digest: general_purpose::STANDARD.encode(digest_dataset), 
            jws: jws.into(),
            did_publisher: did_publisher,
        })

    }

    pub fn verify(&self, iota_document: &IotaDocument) -> anyhow::Result<()> {
        
        if iota_document.verify_jws(
            &Jws::from(self.jws.clone()),
            None,
            &EdDSAJwsVerifier::default(),
            &JwsVerificationOptions::default(),
        ).is_err() {
            return Err(anyhow::anyhow!("Verification Failed"))
        } // TODO: define and catch the error on caller and log ("Signature NOT Valid")

        Ok(())  
    }
}