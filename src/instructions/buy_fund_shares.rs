use solana_program::{
    account_info::AccountInfo, 
    entrypoint::ProgramResult, 
    program_error::ProgramError, 
    pubkey::Pubkey
};

use crate::state::fund_account::FundAccount;

pub fn buy_fund_shares(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount_in_fiat: u64,
    fund_name: String
) -> ProgramResult {

    let [
        punto_xero_master,
        fund_account,
        mint_account,
        buyer,
        buyer_ata,
        system_program,
        token_program,
        associated_token_account_program
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    FundAccount::buy_fund_shares(
        program_id, 
        punto_xero_master, 
        fund_account, 
        mint_account, 
        buyer, 
        buyer_ata, 
        system_program, 
        token_program,
        associated_token_account_program,
        amount_in_fiat, 
        &fund_name
    )?;

    Ok(())
}