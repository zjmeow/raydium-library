use crate::{lock_instructions, lock_utils, LockedCpLiquidityState};
use anyhow::Result;
use clap::Parser;
use common::{common_types, common_utils, rpc, token};
use rand::rngs::OsRng;
use solana_client::{
    rpc_client::RpcClient,
    rpc_filter::{Memcmp, RpcFilterType},
};
use solana_sdk::{
    instruction::Instruction,
    pubkey::Pubkey,
    signer::{keypair::Keypair, Signer},
};
use std::sync::Arc;

#[derive(Debug, Parser)]
pub enum LockCommands {
    LockCpLiquidity {
        #[clap(long)]
        pool_id: Pubkey,
        #[clap(long)]
        lp_token_account: Pubkey,
        #[clap(long)]
        lp_amount: u64,
        #[clap(short, long, action)]
        with_metadata: bool,
    },
    CollectCpFee {
        fee_nft_account: Pubkey,
    },
}

pub fn process_lock_commands(
    command: LockCommands,
    config: &common_types::CommonConfig,
    signing_keypairs: &mut Vec<Arc<dyn Signer>>,
) -> Result<Option<Vec<Instruction>>> {
    let rpc_client = RpcClient::new(config.cluster().url());
    let wallet_keypair = common_utils::read_keypair_file(&config.wallet())?;
    let payer_pubkey = wallet_keypair.pubkey();
    let payer: Arc<dyn Signer> = Arc::new(wallet_keypair);
    if !signing_keypairs.contains(&payer) {
        signing_keypairs.push(payer);
    }

    match command {
        LockCommands::LockCpLiquidity {
            pool_id,
            lp_token_account,
            lp_amount,
            with_metadata,
        } => {
            let pool_state = rpc::get_anchor_account::<raydium_cp_swap::states::PoolState>(
                &rpc_client,
                &pool_id,
            )
            .unwrap()
            .unwrap();
            let fee_nft_mint_keypair = Keypair::generate(&mut OsRng);
            let fee_nft_mint = fee_nft_mint_keypair.pubkey();
            let signer: Arc<dyn Signer> = Arc::new(fee_nft_mint_keypair);
            if !signing_keypairs.contains(&signer) {
                println!("fee_nft_mint:{}", fee_nft_mint);
                signing_keypairs.push(signer);
            }

            let lock_cp_liquidity_instruction = lock_instructions::lock_cp_liquidity_instruction(
                config,
                lp_token_account,
                fee_nft_mint,
                pool_id,
                pool_state.lp_mint,
                pool_state.token_0_vault,
                pool_state.token_1_vault,
                lp_amount,
                with_metadata,
            )
            .unwrap();
            return Ok(Some(lock_cp_liquidity_instruction));
        }
        LockCommands::CollectCpFee { fee_nft_account } => {
            // load locked nft mint
            let fee_nft_token_data = &rpc_client.get_account_data(&fee_nft_account)?;
            let fee_nft_token_info = common_utils::unpack_token(fee_nft_token_data).unwrap();
            let fee_nft_mint = fee_nft_token_info.base.mint;

            let state_data = rpc::get_account(
                &rpc_client,
                &Pubkey::find_program_address(
                    &[
                        lock_utils::LOCKED_LIQUIDITY_SEED.as_bytes(),
                        fee_nft_mint.as_ref(),
                    ],
                    &config.lock_program(),
                )
                .0,
            )
            .unwrap()
            .unwrap();
            let locked_liquidity_state =
                LockedCpLiquidityState::deserialize_account(&state_data.as_slice()).unwrap();
            assert!(fee_nft_mint == locked_liquidity_state.fee_nft_mint);

            let pool_state = rpc::get_anchor_account::<raydium_cp_swap::states::PoolState>(
                &rpc_client,
                &locked_liquidity_state.pool_id,
            )
            .unwrap()
            .unwrap();

            let user_token_0 =
                spl_associated_token_account::get_associated_token_address_with_program_id(
                    &payer_pubkey,
                    &pool_state.token_0_mint,
                    &pool_state.token_0_program,
                );
            let user_token_1 =
                spl_associated_token_account::get_associated_token_address_with_program_id(
                    &payer_pubkey,
                    &pool_state.token_1_mint,
                    &pool_state.token_1_program,
                );

            let collect_cp_fees_instruction = lock_instructions::collect_cp_fees_instruction(
                &config,
                fee_nft_mint,
                fee_nft_account,
                locked_liquidity_state.pool_id,
                pool_state.lp_mint,
                pool_state.token_0_vault,
                pool_state.token_1_vault,
                pool_state.token_0_mint,
                pool_state.token_1_mint,
                user_token_0,
                user_token_1,
                u64::MAX,
            )?;

            return Ok(Some(collect_cp_fees_instruction));
        }
    }
}
