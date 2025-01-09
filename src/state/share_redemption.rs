use solana_program::{
    pubkey::Pubkey,
    program_pack::{IsInitialized, Sealed}, 
    entrypoint::ProgramResult,
    account_info::AccountInfo,
    system_program::ID as SYSTEM_PROGRAM_ID,
    program_error::ProgramError,
    borsh1::try_from_slice_unchecked, 
    program::{invoke_signed, invoke},
    system_instruction::create_account, 
    sysvar::Sysvar,
    rent::Rent,
    clock::Clock, 
};
use spl_token::{
    ID as TOKEN_PROGRAM_ID,
    instruction::{transfer, burn},
};

use borsh::{BorshDeserialize, BorshSerialize};

use crate::{errors::CustomError, utils::fixed_point_multiply_checked};

use super::fund_account::FundAccount;

#[derive(BorshSerialize, BorshDeserialize)]
pub struct ShareRedemption {
    pub is_initialized: bool,
    pub bump_seed: u8,
    pub fund_account: Pubkey,
    pub investor: Pubkey,
    pub shares_amount: u64,
    pub share_value: u64,
    pub created_timestamp: i64,
}

impl Sealed for ShareRedemption {}

impl IsInitialized for ShareRedemption {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl ShareRedemption {

    const LEN: usize = 4
        + 1
        + 1
        + 32
        + 32
        + 8
        + 8;

    pub fn create_share_redemption<'a>(
        program_id: &Pubkey,
        punto_xero: &AccountInfo<'a>,
        fund_account: &AccountInfo<'a>,
        fund_vault: &AccountInfo<'a>,
        mint_account: &AccountInfo<'a>,
        share_redemption_account: &AccountInfo<'a>,
        investor: &AccountInfo<'a>,
        investor_ata: &AccountInfo<'a>,
        token_program: &AccountInfo<'a>,
        system_program: &AccountInfo<'a>,
        fund_name: String,
        shares_to_redeem: u64,
    ) -> ProgramResult {

        if *system_program.key != SYSTEM_PROGRAM_ID {
            return Err(ProgramError::InvalidAccountData)
        }

        if !punto_xero.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }
        
        if !investor.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        if *token_program.key != TOKEN_PROGRAM_ID {
            return Err(ProgramError::InvalidAccountData)
        }

        let fund_account_data = try_from_slice_unchecked::<FundAccount>(
            &fund_account.data.borrow()[..]
        )?;

        let fund_account_pda = Pubkey::create_program_address(
            &[b"fund_account", fund_name.as_bytes(), &[fund_account_data.bump_seed]], 
            program_id
        )?;

        if fund_account_pda != *fund_account.key {
            return Err(ProgramError::InvalidAccountData);
        }

        let (fund_vault_pda, _vault_bump_seed) = Pubkey::find_program_address(
            &[b"fund_vault", fund_name.as_bytes()], 
            program_id
        );

        if fund_vault_pda != *fund_vault.key {
            return Err(ProgramError::InvalidAccountData)
        }

        let (mint_pda, _mint_bump) = Pubkey::find_program_address(
            &[b"fund_mint", fund_name.as_bytes()], 
            program_id
        );

        if mint_pda != *mint_account.key {
            return Err(ProgramError::InvalidAccountData)
        }

        let (share_redemption_pda, share_redemption_bump) = Pubkey::find_program_address(
            &[b"share_redemption", fund_name.as_bytes(), &investor.key.to_bytes()], 
            program_id
        );

        if share_redemption_pda != *share_redemption_account.key {
            return Err(ProgramError::InvalidAccountData);
        }

        let space = Self::LEN;
        let rent = Rent::get()?.minimum_balance(space);
        let current_timestamp = Clock::get()?.unix_timestamp;

        invoke_signed(
            &create_account(
                &punto_xero.key, 
                &share_redemption_account.key, 
                rent, 
                space as u64, 
                program_id
            ), 
            &[
                punto_xero.clone(),
                share_redemption_account.clone(),
                system_program.clone()
            ], 
            &[
                &[
                    b"share_redemption",
                    fund_name.as_bytes(),
                    &investor.key.to_bytes(),
                    &[share_redemption_bump]
                ]
            ]
        )?;

        let mut share_redemption_account_data = try_from_slice_unchecked::<ShareRedemption>(
            &mut &mut share_redemption_account.data.borrow_mut()[..]
        )?;

        if share_redemption_account_data.is_initialized() {
            return Err(ProgramError::AccountAlreadyInitialized);
        };

        share_redemption_account_data.is_initialized = true;
        share_redemption_account_data.bump_seed = share_redemption_bump;
        share_redemption_account_data.fund_account = *fund_account.key;
        share_redemption_account_data.investor = *investor.key;
        share_redemption_account_data.shares_amount = shares_to_redeem;
        share_redemption_account_data.share_value = fund_account_data.share_value;
        share_redemption_account_data.created_timestamp = current_timestamp;

        share_redemption_account_data.serialize(&mut &mut share_redemption_account.data.borrow_mut()[..])?;

        let tranfer_ix = transfer(
            &token_program.key, 
            &investor_ata.key, 
            &fund_vault.key, 
            &investor.key, 
            &[investor.key], 
            shares_to_redeem
        )?;

        invoke(
            &tranfer_ix, 
            &[
                token_program.clone(),
                investor_ata.clone(),
                fund_vault.clone(),
                investor.clone()
            ]
        )?;

        Ok(())
    }

    pub fn process_share_redemption<'a>(
        program_id: &Pubkey,
        punto_xero: &AccountInfo<'a>,
        fund_account: &AccountInfo<'a>,
        mint_account: &AccountInfo<'a>,
        share_redemption_account: &AccountInfo<'a>,
        fund_vault: &AccountInfo<'a>,
        investor: &AccountInfo<'a>,
        token_program: &AccountInfo<'a>,
        system_program: &AccountInfo<'a>,
        amount_payed: u64,
        fund_name: String,
    ) -> ProgramResult {

        if !punto_xero.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        if fund_account.owner != program_id {
            return Err(ProgramError::IllegalOwner);
        }

        let fund_account_data = try_from_slice_unchecked::<FundAccount>(
            &fund_account.data.borrow()[..]
        )?;

        let fund_account_pda = Pubkey::create_program_address(
            &[b"fund_account", fund_name.as_bytes(), &[fund_account_data.bump_seed]], 
            program_id
        )?;

        if fund_account_pda != *fund_account.key {
            return Err(ProgramError::InvalidAccountData);
        }

        if *system_program.key != SYSTEM_PROGRAM_ID {
            return Err(ProgramError::InvalidAccountData)
        }

        if *token_program.key != TOKEN_PROGRAM_ID {
            return Err(ProgramError::InvalidAccountData)
        }

        let (mint_pda, _mint_bump) = Pubkey::find_program_address(
            &[b"fund_mint", fund_name.as_bytes()], 
            program_id
        );

        if mint_pda != *mint_account.key {
            return Err(ProgramError::InvalidAccountData)
        }

        let (fund_vault_pda, _vault_bump_seed) = Pubkey::find_program_address(
            &[b"fund_vault", fund_name.as_bytes()], 
            program_id
        );

        if fund_vault_pda != *fund_vault.key {
            return Err(ProgramError::InvalidAccountData)
        }

        let (share_redemption_pda, _share_redemption_bump) = Pubkey::find_program_address(
            &[b"share_redemption", fund_name.as_bytes(), &investor.key.to_bytes()], 
            program_id
        );

        if share_redemption_pda != *share_redemption_account.key {
            return Err(ProgramError::InvalidAccountData);
        }

        let share_redemption_data 
            = try_from_slice_unchecked::<ShareRedemption>(&mut &mut share_redemption_account.data.borrow_mut()[..])?;

        let amount_to_be_payed = fixed_point_multiply_checked(
            share_redemption_data.share_value,
            share_redemption_data.shares_amount
        )?;

        if amount_to_be_payed != amount_payed {
            return Err(CustomError::InvalidRedemptionAmount.into())
        }

        let burn_ix = burn(
            &token_program.key, 
            &fund_vault.key, 
            &mint_account.key, 
            &fund_account.key, 
            &[fund_account.key], 
            share_redemption_data.shares_amount
        )?;

        invoke_signed(
            &burn_ix, 
            &[
                token_program.clone(),
                fund_vault.clone(),
                mint_account.clone(),
                fund_account.clone(),
            ], 
            &[
                &[
                    b"fund_account",
                    fund_name.as_bytes(),
                    &[fund_account_data.bump_seed]
                ]
            ]
        )?;

        let empty_account_span = 0usize;
        let lamports_required = (Rent::get()?).minimum_balance(empty_account_span);
        let diff = share_redemption_account.lamports() - lamports_required;
        **share_redemption_account.lamports.borrow_mut() -= diff;
        **punto_xero.lamports.borrow_mut() += diff;
        share_redemption_account.realloc(empty_account_span, true)?;
        share_redemption_account.assign(system_program.key);

        Ok(())
    }
}