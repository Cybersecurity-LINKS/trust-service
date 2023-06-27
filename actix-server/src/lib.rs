use iota_client::{secret::SecretManager, Client};
use std::sync::Mutex;

pub mod controllers;
pub mod dtos;
pub mod services;
pub mod utils;

// This struct represents state
pub struct AppIotaState {
    pub app_name: String,
    pub secret_manager: Mutex<SecretManager>,
    pub client: Client,
}