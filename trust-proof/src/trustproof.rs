use crypto::hashes::blake2b::Blake2b256;
use identity_iota::crypto::Ed25519;
use identity_iota::crypto::PublicKey;
use identity_iota::crypto::Sign;

use identity_iota::prelude::KeyPair;
use serde::Serialize;
use serde::Deserialize;
use serde_json::Value;
use crypto::hashes::Digest;
use identity_iota::crypto::Verify;
use base64::{Engine as _, engine::{general_purpose}};
use wasm_bindgen::prelude::*;

// #[wasm_bindgen(js_name = TrustProof, inspectable)]
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct TrustProof {
    offer_digest: String,
    dataset_digest: String,
    signature: String,
    pub did_publisher: String, //TODO: beware of pub
}

// #[wasm_bindgen(js_class = TrustProof)]
impl TrustProof {
    
    // #[wasm_bindgen(constructor)]
    pub fn new(
        offer: &Value,
        dataset: &String,
        key_pair_publisher: &KeyPair,
        did_publisher: String
    ) -> Self {

        // digest_offer = Hash(offerta), digest_dataset = Hash(Dataset), Sign(digest_offer+digest_dataset)

        let digest_offer: [u8; 32] = Blake2b256::digest(offer.to_string().as_bytes()).as_slice().try_into().expect("Wrong length");
        let digest_dataset: [u8; 32]  = Blake2b256::digest(dataset.as_bytes()).as_slice().try_into().expect("Wrong length");

        let digests_sum = [digest_offer, digest_dataset];

        let digests = digests_sum.concat();

        //compute connector signature

        let connector_signature: [u8; Ed25519::SIGNATURE_LENGTH] = Ed25519::sign(&digests, key_pair_publisher.private()).expect("Wrong length");
        
        //verify connector signature

        let valid = Ed25519::verify(&digests, &connector_signature, key_pair_publisher.public());
        
        if valid.is_ok() == false {
            // TODO: handle error
            panic!("Signature NOT Valid");
        }   

        Self{
            offer_digest: general_purpose::STANDARD.encode(digest_offer), 
            dataset_digest: general_purpose::STANDARD.encode(digest_dataset), 
            signature: general_purpose::STANDARD.encode(connector_signature),
            did_publisher: did_publisher,
        }

    }

    // #[wasm_bindgen]
    pub fn verify(&self, publisher_public_key: &PublicKey) -> bool {
        
        let trust_metadata_to_verify = [
            general_purpose::STANDARD.decode(&self.offer_digest).unwrap(), 
            general_purpose::STANDARD.decode(&self.dataset_digest).unwrap()
        ].concat();

        let valid = Ed25519::verify(
            &trust_metadata_to_verify, 
            &general_purpose::STANDARD.decode(&self.signature).unwrap(), 
            publisher_public_key
        );
        
        if valid.is_ok() {
            println!("Trust proof signature is VALID");
            true
        } else {
            println!("Trust proof signature is NOT valid");
            // TODO: handle error
            false
        }   

    }
}