#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use instructions::{BuyFundSharesPayload, FundInitPayload, ProcessSharesRedemptionPayload, RedeemSharesPayload, UpdateShareValuePayload};
    use solana_program_test::{tokio::{self, sync::Mutex}, BanksClient, ProgramTest};
    use solana_sdk::{
        hash::Hash, 
        instruction::{AccountMeta, Instruction}, 
        pubkey::Pubkey, 
        signature::Keypair, 
        signer::Signer, 
        system_program::ID as SYSTEM_PROGRAM_ID, 
        sysvar::rent::ID as RENT_SYSVAR_ID, 
        transaction::Transaction,
        program_pack::Pack
    };
    use spl_associated_token_account::{
        get_associated_token_address,
        ID as ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID
    };
    use spl_token::{
        ID as TOKEN_PROGRAM_ID,
        state::Account as TokenAccount
    };
    use borsh::{BorshDeserialize, BorshSerialize};
    use crate::*;

    struct TestSetup {
        pub program_id: Pubkey,
        pub punto_xero_master: Keypair,
        pub fund_manager_master: Keypair,
        pub buyer: Keypair,
        pub fund_account: Pubkey,
        pub fund_mint: Pubkey,
        pub fund_vault: Pubkey,
        pub banks_client: Arc<Mutex<BanksClient>>,
        pub recent_blockhash: Hash,
        pub fund_name: String
    }

    const SCALE_FACTOR: u64 = 1_000_000;

    async fn get_setup() -> TestSetup {
        solana_logger::setup_with_default("solana_program::message=debug");

        let program_id = Pubkey::new_unique();
        let fund_manager_master = Keypair::new();
        let buyer = Keypair::new();
        let fund_name = String::from("PuntoXero");
        let (fund_account_pda, _fund_bump_seed) = Pubkey::find_program_address(
            &[b"fund_account", fund_name.as_bytes()], 
            &program_id
        );
        let (fund_mint_account, _mint_bump_seed) = Pubkey::find_program_address(
            &[b"fund_mint", fund_name.as_bytes()], 
            &program_id
        );
        let fund_vault = get_associated_token_address(
            &fund_account_pda, 
            &fund_mint_account
        );

        let program_test = ProgramTest::new(
            "solana_program_px", 
            program_id, 
            None
        );

        let (
            banks_client, 
            punto_xero_master, 
            recent_blockhash
        ) = program_test.start().await;

        TestSetup {
            program_id,
            punto_xero_master,
            fund_manager_master,
            buyer,
            fund_account: fund_account_pda,
            fund_mint: fund_mint_account,
            fund_vault,
            banks_client: Arc::new(Mutex::new(banks_client)),
            recent_blockhash,
            fund_name
        }
    }

    async fn initialize_fund_and_mint(setup: &TestSetup) {
        let TestSetup {
            program_id,
            punto_xero_master,
            fund_manager_master,
            buyer: _,
            fund_account,
            fund_mint,
            fund_vault,
            banks_client,
            recent_blockhash,
            fund_name
        } = setup;

        let mut banks_client = banks_client.lock().await;

        let initialize_payload 
            = FundInitPayload { share_value: 10_000 * SCALE_FACTOR, fund_name: fund_name.clone()};

        let mut initialize_payload_data = Vec::new();
        initialize_payload.serialize(&mut initialize_payload_data)
            .unwrap();

        let instruction = Instruction::new_with_bytes(
            *program_id, 
            &[&[0][..], &initialize_payload_data].concat(),
            vec![
                AccountMeta::new(punto_xero_master.pubkey(), true),
                AccountMeta::new_readonly(fund_manager_master.pubkey(), true),
                AccountMeta::new(*fund_account, false),
                AccountMeta::new(*fund_mint, false),
                AccountMeta::new(*fund_vault, false),
                AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
                AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
                AccountMeta::new_readonly(ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID, false),
                AccountMeta::new_readonly(RENT_SYSVAR_ID, false),
            ]
        );

        let transaction = Transaction::new_signed_with_payer(
            &[instruction], 
            Some(&punto_xero_master.pubkey()), 
            &[
                punto_xero_master,
                fund_manager_master
            ], 
            *recent_blockhash
        );

        banks_client.process_transaction(transaction).await.unwrap();
    }

    

    #[tokio::test]
    async fn test_fund_initialization() {
        let setup = get_setup().await;

        initialize_fund_and_mint(&setup).await;

        let fund_account_info = setup.banks_client.lock().await
            .get_account(setup.fund_account)
            .await
            .unwrap()
            .unwrap();

        let fund_account_data = state::fund_account::FundAccount::try_from_slice(&fund_account_info.data)
            .unwrap();

        assert_eq!(&fund_account_data.fund_name, &setup.fund_name);
        assert_eq!(&fund_account_data.fund_mint, &setup.fund_mint);
        assert_eq!(&fund_account_data.fund_vault, &setup.fund_vault);
        assert_eq!(fund_account_data.share_value, 10_000 * SCALE_FACTOR);
    }

    #[tokio::test]
    async fn test_update_share_value() {
        let setup = get_setup().await;

        initialize_fund_and_mint(&setup).await;

        let mut banks_client = setup.banks_client.lock().await;

        let new_share_value = 11000 * SCALE_FACTOR;

        let update_share_value_payload 
            = UpdateShareValuePayload { new_share_value, fund_name: setup.fund_name };

        let mut update_share_value_payload_data = Vec::new();
        update_share_value_payload.serialize(&mut update_share_value_payload_data)
            .unwrap();

        let instruction = Instruction::new_with_bytes(
            setup.program_id, 
            &[&[1][..], &update_share_value_payload_data].concat(),
            vec![
                AccountMeta::new(setup.punto_xero_master.pubkey(), true),
                AccountMeta::new_readonly(setup.fund_manager_master.pubkey(), true),
                AccountMeta::new(setup.fund_account, false),
            ]
        );

        let transaction = Transaction::new_signed_with_payer(
            &[instruction], 
            Some(&setup.punto_xero_master.pubkey()), 
            &[
                setup.punto_xero_master,
                setup.fund_manager_master
            ], 
            setup.recent_blockhash
        );

        banks_client.process_transaction(transaction).await.unwrap();

        let fund_account_info = banks_client
            .get_account(setup.fund_account)
            .await
            .unwrap()
            .unwrap();

        let fund_account_data = state::fund_account::FundAccount::try_from_slice(&fund_account_info.data)
            .unwrap();

        assert_eq!(fund_account_data.share_value, new_share_value);

    }


    async fn buy_shares(
        setup: &TestSetup
    ) {
        let TestSetup {
            program_id,
            punto_xero_master,
            fund_manager_master: _,
            buyer,
            fund_account,
            fund_mint,
            fund_vault: _,
            banks_client,
            recent_blockhash,
            fund_name
        } = setup;

        let mut banks_client = banks_client.lock().await;

        let buyer_ata = get_associated_token_address(&buyer.pubkey(), &fund_mint);

        let buy_shares_payload 
            = BuyFundSharesPayload { amount_in_fiat: 20_000 * SCALE_FACTOR, fund_name: fund_name.clone() };

        let mut buy_shares_payload_data = Vec::new();
        buy_shares_payload.serialize(&mut buy_shares_payload_data)
            .unwrap();

        let instruction = Instruction::new_with_bytes(
            *program_id, 
            &[&[2][..], &buy_shares_payload_data].concat(),
            vec![
                AccountMeta::new(punto_xero_master.pubkey(), true),
                AccountMeta::new(*fund_account, false),
                AccountMeta::new(*fund_mint, false),
                AccountMeta::new(buyer.pubkey(), true),
                AccountMeta::new(buyer_ata, false),
                AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
                AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
                AccountMeta::new_readonly(ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID, false),
            ]
        );

        let transaction = Transaction::new_signed_with_payer(
            &[instruction], 
            Some(&punto_xero_master.pubkey()), 
            &[
                punto_xero_master,
                buyer
            ], 
            *recent_blockhash
        );

        banks_client.process_transaction(transaction).await.unwrap();
    }

    #[tokio::test]
    async fn test_buy_shares() {
        let setup = get_setup().await;

        initialize_fund_and_mint(&setup).await;

        buy_shares(&setup).await;

        let TestSetup {
            program_id: _,
            punto_xero_master: _,
            fund_manager_master: _,
            buyer,
            fund_account: _,
            fund_mint,
            fund_vault: _,
            banks_client,
            recent_blockhash: _,
            fund_name: _
        } = setup;


        let mut banks_client = banks_client.lock().await;

        let buyer_ata = get_associated_token_address(&buyer.pubkey(), &fund_mint);

        let token_acount_info = banks_client
            .get_account(buyer_ata)
            .await
            .unwrap()
            .unwrap();

        let token_account_data = TokenAccount::unpack(&token_acount_info.data)
            .unwrap();

        assert_eq!(token_account_data.amount, 2 * SCALE_FACTOR);

    }

    async fn redeem_shares(setup: &TestSetup) {

        let TestSetup {
            program_id,
            punto_xero_master,
            fund_manager_master: _,
            buyer,
            fund_account,
            fund_mint,
            fund_vault,
            banks_client,
            recent_blockhash,
            fund_name
        } = setup;

        let mut banks_client = banks_client.lock().await;

        let buyer_ata = get_associated_token_address(&buyer.pubkey(), &fund_mint);

        let (buyer_share_redemption, _bump_seed) = Pubkey::find_program_address(
            &[b"share_redemption", fund_name.as_bytes(), &buyer.pubkey().to_bytes()], 
            &program_id
        );

        let share_redemption_payload 
            = RedeemSharesPayload { shares_to_redeem: 1 * SCALE_FACTOR, fund_name: fund_name.clone() };

        let mut payload_data = Vec::new();
        share_redemption_payload.serialize(&mut payload_data)
            .unwrap();

        let instruction = Instruction::new_with_bytes(
            *program_id, 
            &[&[3][..], &payload_data].concat(), 
            vec![
                AccountMeta::new(punto_xero_master.pubkey(), true),
                AccountMeta::new(*fund_account, false),
                AccountMeta::new(*fund_mint, false),
                AccountMeta::new(*fund_vault, false),
                AccountMeta::new(buyer_share_redemption, false),
                AccountMeta::new(buyer.pubkey(), true),
                AccountMeta::new(buyer_ata, false),
                AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
                AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
            ]
        );

        let transaction = Transaction::new_signed_with_payer(
            &[instruction], 
            Some(&punto_xero_master.pubkey()), 
            &[
                punto_xero_master,
                buyer
            ], 
            *recent_blockhash
        );

        banks_client.process_transaction(transaction).await.unwrap();

    }

    #[tokio::test]
    async fn test_redeem_shares() {
        let setup = get_setup().await;

        initialize_fund_and_mint(&setup).await;

        buy_shares(&setup).await;

        redeem_shares(&setup).await;

        let TestSetup {
            program_id: _,
            punto_xero_master: _,
            fund_manager_master: _,
            buyer,
            fund_account,
            fund_mint,
            fund_vault: _,
            banks_client,
            recent_blockhash: _,
            fund_name: _
        } = setup;

        let mut banks_client = banks_client.lock().await;

        let buyer_ata = get_associated_token_address(&buyer.pubkey(), &fund_mint);
        let fund_vault = get_associated_token_address(&fund_account, &fund_mint);

        let buyer_token_acount_info = banks_client
            .get_account(buyer_ata)
            .await
            .unwrap()
            .unwrap();

        let fund_token_acount_info = banks_client
            .get_account(fund_vault)
            .await
            .unwrap()
            .unwrap();

        let buyer_token_account_data = TokenAccount::unpack(&buyer_token_acount_info.data)
            .unwrap();

        let vault_token_account_data = TokenAccount::unpack(&fund_token_acount_info.data)
            .unwrap();

        assert_eq!(buyer_token_account_data.amount, 1 * SCALE_FACTOR);
        assert_eq!(vault_token_account_data.amount, 1 * SCALE_FACTOR);
    }

    async fn process_share_redemption(setup: &TestSetup) {
        let TestSetup {
            program_id,
            punto_xero_master,
            fund_manager_master,
            buyer,
            fund_account,
            fund_mint,
            fund_vault,
            banks_client,
            recent_blockhash,
            fund_name
        } = setup;

        let mut banks_client = banks_client.lock().await;

        let (buyer_share_redemption, _bump_seed) = Pubkey::find_program_address(
            &[b"share_redemption", fund_name.as_bytes(), &buyer.pubkey().to_bytes()], 
            &program_id
        );

        let share_redemption_payload 
            = ProcessSharesRedemptionPayload { amount_payed: 10_000 * SCALE_FACTOR, fund_name: fund_name.clone() };

        let mut payload_data = Vec::new();
        share_redemption_payload.serialize(&mut payload_data)
            .unwrap();

        let instruction = Instruction::new_with_bytes(
            *program_id, 
            &[&[4][..], &payload_data].concat(), 
            vec![
                AccountMeta::new(punto_xero_master.pubkey(), true),
                AccountMeta::new_readonly(fund_manager_master.pubkey(), true),
                AccountMeta::new(*fund_account, false),
                AccountMeta::new(*fund_mint, false),
                AccountMeta::new(*fund_vault, false),
                AccountMeta::new(buyer_share_redemption, false),
                AccountMeta::new(buyer.pubkey(), false),
                AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
                AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
            ]
        );

        let transaction = Transaction::new_signed_with_payer(
            &[instruction], 
            Some(&punto_xero_master.pubkey()), 
            &[
                punto_xero_master,
                fund_manager_master
            ], 
            *recent_blockhash
        );

        banks_client.process_transaction(transaction).await.unwrap();
    }


    #[tokio::test]
    async fn test_process_share_redemption() {
        let setup = get_setup().await;

        initialize_fund_and_mint(&setup).await;

        buy_shares(&setup).await;

        redeem_shares(&setup).await;

        process_share_redemption(&setup).await;

        let TestSetup {
            program_id,
            punto_xero_master: _,
            fund_manager_master: _,
            buyer,
            fund_account,
            fund_mint,
            fund_vault: _,
            banks_client,
            recent_blockhash: _,
            fund_name
        } = setup;

        let mut banks_client = banks_client.lock().await;

        let fund_vault = get_associated_token_address(&fund_account, &fund_mint);

        let (buyer_share_redemption, _bump_seed) = Pubkey::find_program_address(
            &[b"share_redemption", fund_name.as_bytes(), &buyer.pubkey().to_bytes()], 
            &program_id
        );

        let redemption_account = banks_client
            .get_account(buyer_share_redemption)
            .await
            .unwrap()
            .unwrap();

        let fund_token_acount_info = banks_client
            .get_account(fund_vault)
            .await
            .unwrap()
            .unwrap();

        let vault_token_account_data = TokenAccount::unpack(&fund_token_acount_info.data)
            .unwrap();

        assert_eq!(redemption_account.owner, SYSTEM_PROGRAM_ID);
        assert_eq!(redemption_account.data, Vec::<u8>::new());
        assert_eq!(vault_token_account_data.amount, 0 * SCALE_FACTOR);


    }

}