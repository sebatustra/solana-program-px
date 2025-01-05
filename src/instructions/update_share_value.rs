use solana_program::{
    account_info::AccountInfo, 
    entrypoint::ProgramResult, 
    program_error::ProgramError, 
    pubkey::Pubkey
};

use crate::state::fund_account::FundAccount;

pub fn update_share_value(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    new_share_value: u64,
    fund_name: String
) -> ProgramResult {
    let [
        punto_xero_master,
        manager_master,
        fund_account
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    FundAccount::update_share_value(
        program_id, 
        punto_xero_master, 
        manager_master, 
        fund_account, 
        new_share_value,
        fund_name
    )?;
    
    Ok(())
}