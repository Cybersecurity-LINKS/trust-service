use anyhow::Result;
use crate::dtos::proof_dto::ProofRequestDTO;

pub fn create_proof(proof_dto: ProofRequestDTO) -> Result<(),String>  {
    println!("{}", proof_dto.asset_hash);
    // Ok(())
    Err("Error".to_string())
}
