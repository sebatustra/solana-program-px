pub mod create_fund;
pub mod update_share_value;
pub mod buy_fund_shares;
pub mod redeem_shares;
pub mod process_shares_redemption;

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program_error::ProgramError;

pub enum Instructions {
    InitFundAccount { share_value: u64, fund_name: String },
    UpdateShareValue { new_share_value: u64, fund_name: String },
    BuyFundShares { amount_in_fiat: u64, fund_name: String },
    RedeemShares { shares_to_redeem: u64, fund_name: String },
    ProcessSharesRedemption { amount_payed: u64, fund_name: String }
}

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct FundInitPayload {
    pub share_value: u64,
    pub fund_name: String
}

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct UpdateShareValuePayload {
    pub new_share_value: u64,
    pub fund_name: String
}

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct BuyFundSharesPayload {
    pub amount_in_fiat: u64,
    pub fund_name: String
}

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct RedeemSharesPayload {
    pub shares_to_redeem: u64,
    pub fund_name: String
}

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct ProcessSharesRedemptionPayload {
    pub amount_payed: u64,
    pub fund_name: String
}

impl Instructions {
    pub fn unpack(instruction_data: &[u8]) -> Result<Self, ProgramError> {
        let (discriminator, data) = instruction_data
            .split_first()
            .ok_or(ProgramError::InvalidInstructionData)?;

        match discriminator {
            0 => {
                let payload = FundInitPayload::try_from_slice(data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;

                Ok(Self::InitFundAccount {
                    share_value: payload.share_value,
                    fund_name: payload.fund_name
                })
            },
            1 => {
                let payload = UpdateShareValuePayload::try_from_slice(data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;

                Ok(Self::UpdateShareValue {
                    new_share_value: payload.new_share_value,
                    fund_name: payload.fund_name
                })
            },
            2 => {
                let payload = BuyFundSharesPayload::try_from_slice(data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;  

                Ok(Self::BuyFundShares { 
                    amount_in_fiat: payload.amount_in_fiat,
                    fund_name: payload.fund_name 
                })
            },
            3 => {
                let payload = RedeemSharesPayload::try_from_slice(data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;

                Ok(Self::RedeemShares { 
                    shares_to_redeem: payload.shares_to_redeem, 
                    fund_name: payload.fund_name
                }) 
            },
            4 => {
                let payload = ProcessSharesRedemptionPayload::try_from_slice(data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;

                Ok(Self::ProcessSharesRedemption { amount_payed: payload.amount_payed, fund_name: payload.fund_name })
            }
            _ => Err(ProgramError::InvalidInstructionData)
        }
    }
}