// SPDX-FileCopyrightText: 2023 Fondazione LINKS
//
// SPDX-License-Identifier: APACHE-2.0

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProofRequest {
    pub asset_hash: String,
    pub metadata_hash: String,
    pub did: String
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NftRequest {
    pub asset_id: String,
    pub nft_alias: String,
    pub license: String,
    pub did: String
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NftResponse {
    pub asset_id: String,
    pub nft_address: String,
    pub license: String,
    pub did: String
}
