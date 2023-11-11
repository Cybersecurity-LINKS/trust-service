// SPDX-FileCopyrightText: 2023 Fondazione LINKS
//
// SPDX-License-Identifier: APACHE-2.0

use serde::{Serialize, Deserialize};

use super::proof::Proof;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User{
    pub did: String,
    // pub nonce: Vec<u8>,
    // pub private_key:  Vec<u8>,
    pub fragment: String,
    pub proofs: Vec<Proof>
}