
use iota_sdk::client::{Client, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let mnemonic = Client::generate_mnemonic()?;

    println!("Mnemonic: {}", mnemonic.to_string());

    Ok(())
}