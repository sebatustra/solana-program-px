mod instructions;
mod state;
mod utils;
mod errors;
pub mod test;

use instructions::{
    buy_fund_shares::buy_fund_shares, 
    create_fund::init_fund, 
    update_share_value::update_share_value, 
    redeem_shares::redeem_shares,
    process_shares_redemption::process_shares_redemption,
    Instructions
};

use solana_program::{
    entrypoint,
    entrypoint::ProgramResult,
    pubkey::Pubkey,
    account_info::AccountInfo,
};

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    match Instructions::unpack(instruction_data)? {
        Instructions::InitFundAccount {
            share_value, 
            fund_name
        } => init_fund(program_id, accounts, fund_name, share_value),
        Instructions::UpdateShareValue {
            new_share_value,
            fund_name
        } => update_share_value(program_id, accounts, new_share_value, fund_name),
        Instructions::BuyFundShares { 
            amount_in_fiat, 
            fund_name 
        } => buy_fund_shares(program_id, accounts, amount_in_fiat, fund_name),
        Instructions::RedeemShares { 
            shares_to_redeem, 
            fund_name 
        } => redeem_shares(program_id, accounts, shares_to_redeem, fund_name),
        Instructions::ProcessSharesRedemption { 
            amount_payed,
            fund_name 
        } => process_shares_redemption(program_id, accounts, amount_payed, fund_name),
    }
}