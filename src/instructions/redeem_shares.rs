use solana_program::{
    account_info::AccountInfo, 
    entrypoint::ProgramResult, 
    program_error::ProgramError, 
    pubkey::Pubkey,
};

use crate::state::share_redemption::ShareRedemption;

pub fn redeem_shares(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    shares_to_redeem: u64,
    fund_name: String,
) -> ProgramResult {

    let [
        punto_xero_master,
        fund_account,
        mint_account,
        share_redemption_account,
        share_redemption_ata,
        investor,
        investor_ata,
        token_program,
        system_program,
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    ShareRedemption::create_share_redemption(
        program_id, 
        punto_xero_master, 
        fund_account, 
        mint_account, 
        share_redemption_account, 
        share_redemption_ata, 
        investor, 
        investor_ata, 
        token_program, 
        system_program, 
        fund_name, 
        shares_to_redeem
    )?;

    Ok(())
}