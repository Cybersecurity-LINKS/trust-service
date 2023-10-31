use std::env;
use lazy_static::lazy_static;
use serde_json::{json, Value};
use anyhow::{Result, Ok};
use uuid::Uuid;
use base64::{Engine as _, engine::general_purpose};
use rand::Rng;

lazy_static! {
    static ref _KEYCLOAK_URL: String = env::var("KEYCLOAK_URL").unwrap_or("https://moderatekc.test.ctic.es/".to_string());
    static ref _KEYCLOAK_ADMIN_USER: String = env::var("KEYCLOAK_ADMIN_USER").unwrap_or("admin".to_string());
    static ref _KEYCLOAK_ADMIN_PASS: String = env::var("KEYCLOAK_ADMIN_PASS").unwrap_or("SuperModerateSecret2023".to_string());
}

#[derive(Clone)]
pub struct KeycloakSession {
    pub admin_token: String,
    pub realm_name: String,
    pub client_data: Value,
}

pub async fn get_admin_token (http_client: &reqwest::Client) -> Result<String> {
    let token_url = format!("{}realms/master/protocol/openid-connect/token", *_KEYCLOAK_URL);

    let token_data = json!({
        "grant_type": "password",
        "client_id": "admin-cli",
        "username": *_KEYCLOAK_ADMIN_USER,
        "password": *_KEYCLOAK_ADMIN_PASS,
    });

    let response: Value = http_client
        .post(token_url)
        .form(&token_data)
        .send()
        .await?
        .json()
        .await?;

    let access_token = response["access_token"].as_str().unwrap().to_string();

    log::info!("Admin token: {}", access_token);

    Ok(access_token)
}

pub async fn create_realm (http_client: &reqwest::Client, admin_token: &String) -> Result<String> {
    let realm_name = format!("realm-{}",  &Uuid::new_v4().as_simple().to_string().as_str()[..6] );
    let realm_data = json!({"realm": realm_name, "enabled": true});

    let url = format!("{}admin/realms", *_KEYCLOAK_URL);
    
    let response = http_client
        .post(url)
        .bearer_auth(&admin_token)
        .json(&realm_data)
        .send()
        .await?;

    response.error_for_status()?;

    Ok(realm_name)
}

pub async fn create_client (http_client: &reqwest::Client, realm_name: &String, admin_token: &String) -> Result<Value> {

    let client_name = format!("client-{}",  &Uuid::new_v4().as_simple().to_string().as_str()[..6] );

    let client_data = json!({
        "clientId": client_name,
        "name": client_name,
        "enabled": true,
        "protocol": "openid-connect",
    });

    let url = format!("{}admin/realms/{realm_name}/clients", *_KEYCLOAK_URL);

    let response = http_client
        .post(url)
        .bearer_auth(&admin_token)
        .json(&client_data)
        .send()
        .await?;

    response.error_for_status()?;

    Ok(client_data)
}

pub async fn create_user (keycloak_session: &KeycloakSession) -> Result<Value> {
    let http_client = reqwest::Client::new();
    let mut username = format!("client-{}",  &Uuid::new_v4().as_simple().to_string().as_str()[..6] );

    let user_data = json!({
        "username": username,
        "email": username.push_str("@moderate.eu"),
        "firstName": "New",
        "lastName": "User",
        "enabled": true,
    });

    let url = format!("{}admin/realms/{}/users", *_KEYCLOAK_URL, &keycloak_session.realm_name);

    let response = http_client
        .post(url)
        .bearer_auth(&keycloak_session.admin_token)
        .json(&user_data)
        .send()
        .await?;

    response.error_for_status()?;

    Ok(user_data)
}

pub async fn update_user_attrs(keycloak_session: &KeycloakSession, username: &String, additional_attrs: &Value) -> Result<Value> {
    let http_client = reqwest::Client::new();

    // Get the user ID from the username.
    let users_url = format!("{}admin/realms/{}/users", *_KEYCLOAK_URL, &keycloak_session.realm_name);
    let users_params = json!({"username": username});
    let users_response = http_client
        .get(users_url)
        .bearer_auth(&keycloak_session.admin_token)
        .query(&users_params)
        .send()
        .await?;
    
    let users_json: Value = users_response.json().await?;
    assert_eq!( users_json.as_array().unwrap().len(), 1);
    let user_id = users_json[0]["id"].as_str().unwrap().to_string();

    // Get the data of the user specified by the user ID.
    let user_url = format!("{}admin/realms/{}/users/{user_id}", *_KEYCLOAK_URL, &keycloak_session.realm_name);
    let user_response = http_client
        .get(&user_url)
        .bearer_auth(&keycloak_session.admin_token)
        .send()
        .await?;
    
    
    let mut user_data: Value = user_response.error_for_status()?.json().await?;
    log::info!("User data:\n{:#}", user_data);

    // Update the user data with the additional attributes.
    user_data = user_data["attributes"].take();
    user_data["attributes"] = additional_attrs.to_owned();

    let update_user_response = http_client
        .put(&user_url)
        .bearer_auth(&keycloak_session.admin_token)
        .json(&user_data)
        .send()
        .await?;

    update_user_response.error_for_status()?;


    Ok(additional_attrs.to_owned())
}


