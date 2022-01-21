pub mod instruction;
pub mod processor;
pub mod state;
pub mod utils;

use solana_program::{
    account_info::AccountInfo,
    entrypoint,
    entrypoint::ProgramResult,
    pubkey::Pubkey,
};

entrypoint!(process_instruction);

solana_program::declare_id!("Hk5f9Xw9PdaQ9GEg8TPVFusojLA9otDpUkziXw1hAVE5");

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    crate::processor::process_instruction(program_id, accounts, instruction_data)
}
