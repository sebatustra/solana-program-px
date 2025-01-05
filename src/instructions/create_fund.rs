use solana_program::{
    account_info::AccountInfo, 
    entrypoint::ProgramResult, 
    program_error::ProgramError, 
    pubkey::Pubkey,
};

use crate::state::fund_account::FundAccount;

pub fn init_fund(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    fund_name: String,
    share_value: u64
) -> ProgramResult {
    let [
        punto_xero_master, 
        manager_master, 
        fund_account,
        mint_account,
        system_program,
        token_program,
        rent_sysvar,
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    FundAccount::initialize_fund(
        program_id, 
        punto_xero_master, 
        manager_master, 
        fund_account,
        system_program,
        share_value, 
        &fund_name
    )?;

    FundAccount::initialize_fund_mint(
        program_id, 
        punto_xero_master, 
        manager_master, 
        fund_account, 
        mint_account, 
        system_program, 
        token_program,
        rent_sysvar,
        &fund_name
    )?;

    Ok(())
}