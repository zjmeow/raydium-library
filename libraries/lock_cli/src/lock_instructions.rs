use crate::lock_utils::*;
use anchor_lang::prelude::AccountMeta;
use anchor_spl::memo::ID as MEMO_ID;
use anchor_spl::metadata::ID as mpl_token_metadata_id;
use anyhow::{format_err, Result};
use common::common_types::CommonConfig;
use solana_sdk::{instruction::Instruction, pubkey::Pubkey, signer::Signer, system_program};

const METADATA_PREFIX: &str = "metadata";

pub fn lock_cp_liquidity_instruction(
    config: &CommonConfig,
    user_lp_token: Pubkey,
    fee_nft_mint: Pubkey,
    pool_id: Pubkey,
    lp_mint: Pubkey,
    token_0_vault: Pubkey,
    token_1_vault: Pubkey,
    lp_amount: u64,
    with_metadata: bool,
) -> Result<Vec<Instruction>> {
    let wallet = solana_sdk::signature::read_keypair_file(config.wallet())
        .map_err(|_| format_err!("failed to read keypair from {}", config.wallet()))?;
    let program_lock_cp_authority =
        Pubkey::find_program_address(&[LOCK_CP_AUTH_SEED.as_bytes()], &config.lock_program()).0;
    let locked_liquidity = Pubkey::find_program_address(
        &[LOCKED_LIQUIDITY_SEED.as_bytes(), fee_nft_mint.as_ref()],
        &config.lock_program(),
    )
    .0;
    let (metadata_account_key, _bump) = Pubkey::find_program_address(
        &[
            METADATA_PREFIX.as_bytes(),
            mpl_token_metadata_id.to_bytes().as_ref(),
            fee_nft_mint.to_bytes().as_ref(),
        ],
        &mpl_token_metadata_id,
    );
    let fee_nft_account =
        spl_associated_token_account::get_associated_token_address(&wallet.pubkey(), &fee_nft_mint);
    let locked_lp_vault = spl_associated_token_account::get_associated_token_address(
        &program_lock_cp_authority,
        &lp_mint,
    );

    let instruction_data =
        LockCpLiquidityInstruction::serialize_ix(lp_amount, with_metadata).unwrap();
    let accounts = vec![
        AccountMeta::new_readonly(program_lock_cp_authority, false),
        AccountMeta::new(wallet.pubkey(), true),
        AccountMeta::new(wallet.pubkey(), true),
        AccountMeta::new_readonly(wallet.pubkey(), false),
        AccountMeta::new(fee_nft_mint, true),
        AccountMeta::new(fee_nft_account, false),
        AccountMeta::new(pool_id, false),
        AccountMeta::new(locked_liquidity, false),
        AccountMeta::new(lp_mint, false),
        AccountMeta::new(user_lp_token, false),
        AccountMeta::new(locked_lp_vault, false),
        AccountMeta::new(token_0_vault, false),
        AccountMeta::new(token_1_vault, false),
        AccountMeta::new(metadata_account_key, false),
        AccountMeta::new_readonly(solana_sdk::sysvar::rent::id(), false),
        AccountMeta::new_readonly(system_program::id(), false),
        AccountMeta::new_readonly(anchor_spl::token::ID, false),
        AccountMeta::new_readonly(mpl_token_metadata_id, false),
    ];
    let instruction = Instruction {
        program_id: config.lock_program(),
        accounts,
        data: instruction_data,
    };
    Ok(vec![instruction])
}

pub fn collect_cp_fees_instruction(
    config: &CommonConfig,
    fee_nft_mint: Pubkey,
    fee_nft_account: Pubkey,
    pool_id: Pubkey,
    lp_mint: Pubkey,
    token_0_vault: Pubkey,
    token_1_vault: Pubkey,
    vault_0_mint: Pubkey,
    vault_1_mint: Pubkey,
    user_token_0: Pubkey,
    user_token_1: Pubkey,
    fee_lp_amount: u64,
) -> Result<Vec<Instruction>> {
    let wallet = solana_sdk::signature::read_keypair_file(config.wallet())
        .map_err(|_| format_err!("failed to read keypair from {}", config.wallet()))?;

    let program_lock_cp_authority =
        Pubkey::find_program_address(&[LOCK_CP_AUTH_SEED.as_bytes()], &config.lock_program()).0;
    let cp_program_authority = Pubkey::find_program_address(
        &[raydium_cp_swap::AUTH_SEED.as_bytes()],
        &config.cp_program(),
    )
    .0;

    let locked_liquidity = Pubkey::find_program_address(
        &[LOCKED_LIQUIDITY_SEED.as_bytes(), fee_nft_mint.as_ref()],
        &config.lock_program(),
    )
    .0;

    let instruction_data = CollectCpFeeInstruction::serialize_ix(fee_lp_amount).unwrap();
    let accounts = vec![
        AccountMeta::new_readonly(program_lock_cp_authority, false),
        AccountMeta::new(wallet.pubkey(), true),
        AccountMeta::new(wallet.pubkey(), true),
        AccountMeta::new_readonly(wallet.pubkey(), false),
        AccountMeta::new(fee_nft_mint, true),
        AccountMeta::new(fee_nft_account, false),
        AccountMeta::new(locked_liquidity, false),
        AccountMeta::new(config.cp_program(), false),
        AccountMeta::new(cp_program_authority, false),
        AccountMeta::new(pool_id, false),
        AccountMeta::new(lp_mint, false),
        AccountMeta::new(user_token_0, false),
        AccountMeta::new(user_token_1, false),
        AccountMeta::new(token_0_vault, false),
        AccountMeta::new(token_1_vault, false),
        AccountMeta::new_readonly(vault_0_mint, false),
        AccountMeta::new_readonly(vault_1_mint, false),
        AccountMeta::new(
            spl_associated_token_account::get_associated_token_address(
                &program_lock_cp_authority,
                &lp_mint,
            ),
            false,
        ),
        AccountMeta::new_readonly(anchor_spl::token::ID, false),
        AccountMeta::new_readonly(anchor_spl::token_2022::ID, false),
        AccountMeta::new_readonly(MEMO_ID, false),
    ];
    let instruction = Instruction {
        program_id: config.lock_program(),
        accounts,
        data: instruction_data,
    };
    Ok(vec![instruction])
}

pub fn lock_clmm_position_instruction(
    config: &CommonConfig,
    position_nft_account: Pubkey,
    position_nft_mint: Pubkey,
    fee_nft_mint: Pubkey,
    personal_position: Pubkey,
    with_metadata: bool,
) -> Result<Vec<Instruction>> {
    let wallet = solana_sdk::signature::read_keypair_file(config.wallet())
        .map_err(|_| format_err!("failed to read keypair from {}", config.wallet()))?;

    let program_lock_clmm_authority =
        Pubkey::find_program_address(&[LOCK_CLMM_AUTH_SEED.as_bytes()], &config.clmm_program()).0;

    let locked_position = Pubkey::find_program_address(
        &[LOCKED_POSITION_SEED.as_bytes(), fee_nft_mint.as_ref()],
        &config.clmm_program(),
    )
    .0;
    let (metadata_account_key, _bump) = Pubkey::find_program_address(
        &[
            METADATA_PREFIX.as_bytes(),
            mpl_token_metadata_id.to_bytes().as_ref(),
            fee_nft_mint.to_bytes().as_ref(),
        ],
        &mpl_token_metadata_id,
    );
    let fee_nft_account =
        spl_associated_token_account::get_associated_token_address(&wallet.pubkey(), &fee_nft_mint);
    let locked_nft_account = spl_associated_token_account::get_associated_token_address(
        &program_lock_clmm_authority,
        &position_nft_mint,
    );

    let instruction_data = LockClmmPositionInstruction::serialize_ix(with_metadata).unwrap();

    let accounts = vec![
        AccountMeta::new_readonly(program_lock_clmm_authority, false),
        AccountMeta::new(wallet.pubkey(), true),
        AccountMeta::new(wallet.pubkey(), true),
        AccountMeta::new_readonly(wallet.pubkey(), false),
        AccountMeta::new(position_nft_account, false),
        AccountMeta::new(personal_position, false),
        AccountMeta::new(position_nft_mint, false),
        AccountMeta::new(locked_nft_account, false),
        AccountMeta::new(locked_position, false),
        AccountMeta::new(fee_nft_mint, true),
        AccountMeta::new(fee_nft_account, false),
        AccountMeta::new(metadata_account_key, false),
        AccountMeta::new_readonly(mpl_token_metadata_id, false),
        AccountMeta::new_readonly(anchor_spl::associated_token::ID, false),
        AccountMeta::new_readonly(solana_sdk::sysvar::rent::id(), false),
        AccountMeta::new_readonly(anchor_spl::token::ID, false),
        AccountMeta::new_readonly(system_program::id(), false),
    ];
    let instruction = Instruction {
        program_id: config.lock_program(),
        accounts,
        data: instruction_data,
    };
    Ok(vec![instruction])
}
