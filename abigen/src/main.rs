// SPDX-FileCopyrightText: 2024 Fondazione LINKS
//
// SPDX-License-Identifier: Apache-2.0

use std::path::Path;
use eyre::Result;
use ethers::contract::Abigen;
use clap::Parser;

/// Simple program to generate bindings for a smart contract
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the contract ABI JSON file to generate bindings
    #[arg(short, long, required=true)]
    abi_source: String,

    /// Contract name (expected to be CamelCase)
    #[arg(short, long, required=true)]
    contract: String,
}


fn main() -> Result<()> {

    // Parse command line arguments
    let args = Args::parse();
    let abi_source = args.abi_source.as_str();
    let contract = args.contract.as_str();
    println!("{}",contract.to_lowercase());
    let out_file = Path::new("../actix-server/src/contracts/").join(format!("{}.rs", contract.to_lowercase()));
    println!("Generating bindings for contract {} from ABI file {} to {}", contract, abi_source, out_file.display());
    if out_file.exists() {
        std::fs::remove_file(&out_file)?;
    }
    // Generate contract bindings using abigen
    Abigen::new(contract, abi_source)?.generate()?.write_to_file(out_file)?;

    Ok(())
}
