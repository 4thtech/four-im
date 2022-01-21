//! Account utility functions

use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    program::{invoke_signed},
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
};

/// Creates Program Derived Address for the given seeds
pub fn create_pda_account<'a>(
    payer: &AccountInfo<'a>,
    rent: &Rent,
    is_rent_exempt: bool,
    space: usize,
    owner: &Pubkey,
    system_program: &AccountInfo<'a>,
    new_pda_account: &AccountInfo<'a>,
    new_pda_signer_seeds: &[&[u8]],
) -> ProgramResult {
    let mut rent_balance = rent.minimum_balance(space);

    if !is_rent_exempt {
        rent_balance = (rent_balance / (rent.exemption_threshold * 365 as f64) as u64) * 7;
    }

    invoke_signed(
        &system_instruction::create_account(
            payer.key,
            new_pda_account.key,
            1.max(rent_balance),
            space as u64,
            owner,
        ),
        &[
            payer.clone(),
            new_pda_account.clone(),
            system_program.clone(),
        ],
        &[new_pda_signer_seeds],
    )
}
