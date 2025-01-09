use solana_program::{
    account_info::AccountInfo, 
    entrypoint::ProgramResult, 
    program_error::ProgramError, 
    pubkey::Pubkey,
};

use crate::state::share_redemption::ShareRedemption;

pub fn process_shares_redemption(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount_payed: u64,
    fund_name: String,
) -> ProgramResult {

    let [
        punto_xero_master,
        fund_account,
        mint_account,
        share_redemption_account,
        share_redemption_ata,
        investor,
        token_program,
        system_program
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    ShareRedemption::process_share_redemption(
        program_id, 
        punto_xero_master, 
        fund_account, 
        mint_account, 
        share_redemption_account, 
        share_redemption_ata, 
        investor, 
        token_program, 
        system_program, 
        amount_payed,
        fund_name
    )?;

    Ok(())
}