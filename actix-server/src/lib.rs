use iota_client::{secret::SecretManager, Client};
use iota_wallet::account_manager::AccountManager;
use std::sync::{Mutex, RwLock, Arc};

pub mod controllers;
pub mod dtos;
pub mod services;
pub mod utils;

// This struct represents state
pub struct AppIotaState {
    // pub secret_manager: Mutex<&'a mut SecretManager>
    pub account_manager: Arc<RwLock<AccountManager>>
}