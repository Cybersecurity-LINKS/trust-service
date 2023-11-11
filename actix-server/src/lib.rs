// SPDX-FileCopyrightText: 2023 Fondazione LINKS
//
// SPDX-License-Identifier: APACHE-2.0

use iota_sdk::Wallet;
use utils::MemStorage;
use std::sync::{RwLock, Arc};

pub mod controllers;
pub mod dtos;
pub mod services;
pub mod models;
pub mod utils;
pub mod errors;

pub const PROOF_TAG: &str = "proofs-2"; // TODO: define this somewhere else
pub const DB_NAME: &str = "MODERATE"; // TODO: define this somewhere else
pub const USER_COLL_NAME: &str = "Users"; // TODO: define this somewhere else
pub const MAIN_ACCOUNT: &str = "main-account";

// This struct represents the state of the service
pub struct AppIotaState {
    pub wallet: Arc<RwLock<Wallet>>,
    pub key_storage: Arc<RwLock<MemStorage>>
}