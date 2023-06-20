use anyhow::Result;
use crate::dtos::proof_dto::ProofRequestDTO;

pub fn create_proof(proof_dto: ProofRequestDTO) -> Result<()>  {
    println!("{}", proof_dto.asset_hash);
    Ok(())
}
