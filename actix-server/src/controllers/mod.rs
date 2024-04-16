// SPDX-FileCopyrightText: 2023 Fondazione LINKS
//
// SPDX-License-Identifier: APACHE-2.0

pub mod proof_controller;
pub mod did_controller;
pub mod nft_controller;

use serde::Deserialize;


#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AssetQuery {
    asset_id: String,
}