use solana_program::{
    account_info::AccountInfo, 
    borsh1::try_from_slice_unchecked, 
    clock::Clock, 
    entrypoint::ProgramResult, 
    program::{invoke, invoke_signed}, 
    program_error::ProgramError, 
    program_pack::{IsInitialized, Pack, Sealed}, 
    pubkey::Pubkey, 
    rent::Rent, 
    system_instruction::create_account, 
    system_program::ID as SYSTEM_PROGRAM_ID, 
    sysvar::Sysvar,
    msg,
};
use spl_token::{
    ID as TOKEN_PROGRAM_ID,
    instruction::{initialize_mint, mint_to},
    state::Mint
};
use spl_associated_token_account::{
    get_associated_token_address, 
    instruction::create_associated_token_account,
    ID as ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID
};


use borsh::{BorshDeserialize, BorshSerialize};

use crate::utils::fixed_point_divide_checked;
 
#[derive(BorshSerialize, BorshDeserialize)]
pub struct FundAccount {
    pub is_initialized: bool,
    pub bump_seed: u8,
    pub punto_xero_master_pubkey: Pubkey,
    pub manager_master_pubkey: Pubkey,
    pub fund_mint: Pubkey,
    pub mint_bump_seed: u8,
    pub fund_vault: Pubkey,
    pub share_value: u64,
    pub share_value_update: i64,
    pub fund_name: String
}

impl Sealed for FundAccount {}

impl IsInitialized for FundAccount {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl FundAccount {

    fn get_space(name: &str) -> usize {
        return 4
            + 1      // for is_initialized
            + 1      // for bump_seed
            + 32     // for punto_xero_master_pubkey
            + 32     // for manager_master_pubkey
            + 32
            + 1
            + 32
            + 8      // for share_value
            + 8      // for share_value_update 
            + name.len()
    }

    pub fn initialize_fund<'a>(
        program_id: &Pubkey,
        punto_xero: &AccountInfo<'a>,
        manager: &AccountInfo<'a>,
        fund_account: &AccountInfo<'a>,
        mint_account: &AccountInfo<'a>,
        fund_vault: &AccountInfo<'a>,
        system_program: &AccountInfo<'a>,
        token_program: &AccountInfo<'a>,
        associated_token_account_program: &AccountInfo<'a>,
        rent_sysvar: &AccountInfo<'a>,
        share_value: u64,
        fund_name: &str
    ) -> ProgramResult {

        if *system_program.key != SYSTEM_PROGRAM_ID {
            return Err(ProgramError::InvalidAccountData)
        }

        if *token_program.key != TOKEN_PROGRAM_ID {
            return Err(ProgramError::InvalidAccountData)
        }

        if *associated_token_account_program.key != ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID {
            return Err(ProgramError::InvalidAccountData)
        }

        if !punto_xero.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }
        
        if !manager.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let space = Self::get_space(&fund_name);
        let rent = Rent::get()?.minimum_balance(space);
        let current_timestamp = Clock::get()?.unix_timestamp;

        let (pda, bump_seed) = Pubkey::find_program_address(
            &[b"fund_account", fund_name.as_bytes()], 
            program_id
        );

        if pda != *fund_account.key {
            return Err(ProgramError::InvalidAccountData);
        }

        let (mint_pda, mint_bump) = Pubkey::find_program_address(
            &[b"fund_mint", fund_name.as_bytes()], 
            program_id
        );

        if mint_pda != *mint_account.key {
            return Err(ProgramError::InvalidAccountData)
        }

        let fund_vault_pda = get_associated_token_address(
            &fund_account.key, 
            &mint_account.key
        );

        if fund_vault_pda != *fund_vault.key {
            return Err(ProgramError::InvalidAccountData)
        }

        invoke_signed(
            &create_account(
                punto_xero.key, 
                fund_account.key, 
                rent, 
                space as u64, 
                program_id
            ), 
            &[
                punto_xero.clone(),
                fund_account.clone(),
                system_program.clone()
            ], 
            &[
                &[
                    b"fund_account",
                    fund_name.as_bytes(),
                    &[bump_seed]
                ],
            ]
        )?;

        let mut account_data = try_from_slice_unchecked::<FundAccount>(
            &mut &mut fund_account.data.borrow_mut()[..]
        )?;

        if account_data.is_initialized() {
            return Err(ProgramError::AccountAlreadyInitialized);
        };

        account_data.is_initialized = true;
        account_data.bump_seed = bump_seed;
        account_data.punto_xero_master_pubkey = *punto_xero.key;
        account_data.manager_master_pubkey = *manager.key;
        account_data.fund_mint = *mint_account.key;
        account_data.mint_bump_seed = mint_bump;
        account_data.fund_vault = *fund_vault.key;
        account_data.share_value = share_value;
        account_data.share_value_update = current_timestamp;
        account_data.fund_name = fund_name.to_owned();

        account_data.serialize(&mut &mut fund_account.data.borrow_mut()[..])?;

        let mint_space = Mint::LEN;
        let rent_lamports = Rent::get()?.minimum_balance(mint_space);

        let create_account_ix = create_account(
            &punto_xero.key, 
            &mint_account.key, 
            rent_lamports, 
            mint_space as u64, 
            &token_program.key
        );

        invoke_signed(
            &create_account_ix, 
            &[
                punto_xero.clone(),
                mint_account.clone(),
                token_program.clone(),
                system_program.clone()
            ], 
            &[&[
                b"fund_mint", 
                fund_name.as_bytes(), 
                &[mint_bump]
            ]]
        )?;

        let create_mint_ix = initialize_mint(
            &token_program.key, 
            &mint_account.key, 
            &fund_account.key, 
            None, 
            6
        )?;

        invoke_signed(
            &create_mint_ix, 
            &[
                token_program.clone(),
                mint_account.clone(),
                fund_account.clone(),
                rent_sysvar.clone(),
            ], 
            &[&[
                b"fund_mint", 
                fund_name.as_bytes(), 
                &[mint_bump]
            ]]
        )?;

        msg!("created mint!");

        let create_vault_ix = create_associated_token_account(
            punto_xero.key, 
            fund_account.key, 
            mint_account.key, 
            token_program.key
        );

        invoke_signed(
            &create_vault_ix, 
            &[
                mint_account.clone(),
                fund_vault.clone(),
                punto_xero.clone(),
                fund_account.clone(),
                token_program.clone(),
                system_program.clone(),
                associated_token_account_program.clone(),
            ],
            &[
                &[
                    b"fund_account",
                    fund_name.as_bytes(),
                    &[bump_seed]
                ]
            ]
        )?;

        Ok(())
    }

    pub fn update_share_value<'a>(
        program_id: &Pubkey,
        punto_xero: &AccountInfo<'a>,
        manager: &AccountInfo<'a>,
        fund_account: &AccountInfo<'a>,
        new_share_value: u64,
        fund_name: String
    ) -> ProgramResult {
        if !punto_xero.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }
        
        if !manager.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        if fund_account.owner != program_id {
            return Err(ProgramError::IllegalOwner);
        }

        let mut account_data = try_from_slice_unchecked::<FundAccount>(
            &mut &mut fund_account.data.borrow_mut()[..]
        )?;

        let pda = Pubkey::create_program_address(
            &[b"fund_account", fund_name.as_bytes(), &[account_data.bump_seed]], 
            program_id
        )?;

        if pda != *fund_account.key {
            return Err(ProgramError::InvalidAccountData);
        }

        if account_data.punto_xero_master_pubkey != *punto_xero.key {
            return Err(ProgramError::MissingRequiredSignature);
        }

        if account_data.manager_master_pubkey != *manager.key {
            return Err(ProgramError::MissingRequiredSignature);
        }

        account_data.share_value = new_share_value;
        account_data.share_value_update = Clock::get()?.unix_timestamp;

        account_data.serialize(&mut &mut fund_account.data.borrow_mut()[..])?;

        Ok(())
    }

    pub fn buy_fund_shares<'a>(
        program_id: &Pubkey,
        punto_xero: &AccountInfo<'a>,
        fund_account: &AccountInfo<'a>,
        mint_account: &AccountInfo<'a>,
        buyer: &AccountInfo<'a>,
        buyer_ata: &AccountInfo<'a>,
        system_program: &AccountInfo<'a>,
        token_program: &AccountInfo<'a>,
        associated_token_account_program: &AccountInfo<'a>,
        amount_in_fiat: u64,
        fund_name: &str
    ) -> ProgramResult {
        if !punto_xero.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        if !buyer.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        if *token_program.key != TOKEN_PROGRAM_ID {
            return Err(ProgramError::InvalidAccountData)
        }

        if *system_program.key != SYSTEM_PROGRAM_ID {
            return Err(ProgramError::InvalidAccountData)
        }

        if *associated_token_account_program.key != ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID {
            return Err(ProgramError::InvalidAccountData)
        }
        
        let fund_account_data = try_from_slice_unchecked::<FundAccount>(
            &fund_account.data.borrow()[..]
        )?;

        let pda = Pubkey::create_program_address(
            &[b"fund_account", fund_name.as_bytes(), &[fund_account_data.bump_seed]], 
            program_id
        )?;

        if pda != *fund_account.key {
            return Err(ProgramError::InvalidAccountData);
        }

        if fund_account_data.punto_xero_master_pubkey != *punto_xero.key {
            return Err(ProgramError::MissingRequiredSignature);
        }

        if fund_account_data.fund_mint != *mint_account.key {
            return Err(ProgramError::InvalidAccountData);
        }

        let current_share_value = fund_account_data.share_value;

        let shares_to_buy = fixed_point_divide_checked(
            amount_in_fiat, 
            current_share_value
        )?;

        let ata_address = get_associated_token_address(
            &buyer.key, 
            &mint_account.key
        );

        if ata_address != *buyer_ata.key {
            return Err(ProgramError::InvalidAccountData);
        }

        if buyer_ata.lamports() == 0 {

            let create_ata_ix = create_associated_token_account(
                punto_xero.key, 
                buyer.key, 
                mint_account.key, 
                token_program.key
            );

            invoke(
                &create_ata_ix, 
                &[
                    mint_account.clone(),
                    buyer_ata.clone(),
                    punto_xero.clone(),
                    buyer.clone(),
                    token_program.clone(),
                    system_program.clone(),
                    associated_token_account_program.clone(),
                ]
            )?;

        }

        let mint_shares_ix = mint_to(
            &token_program.key, 
            &mint_account.key, 
            &buyer_ata.key, 
            &fund_account.key, 
            &[&fund_account.key], 
            shares_to_buy
        )?;

        invoke_signed(
            &mint_shares_ix, 
            &[
                token_program.clone(),
                mint_account.clone(),
                buyer_ata.clone(),
                buyer.clone(),
                fund_account.clone()
            ], 
            &[
                &[
                    b"fund_account",
                    fund_name.as_bytes(),
                    &[fund_account_data.bump_seed]
                ],
            ]
        )?;
        
        Ok(())
    }
    
}