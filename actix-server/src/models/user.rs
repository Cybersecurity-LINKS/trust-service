use serde::{Serialize, Deserialize};

use super::proof::Proof;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User{
    pub did: String,
    pub nonce: Vec<u8>,
    pub private_key:  Vec<u8>,
    pub proofs: Vec<Proof>
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectedUser {
    pub proof: Proof
}