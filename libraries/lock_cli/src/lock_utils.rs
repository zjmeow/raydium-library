use anchor_lang::prelude::*;
use borsh::{BorshDeserialize, BorshSerialize};

pub const LOCK_CLMM_AUTH_SEED: &str = "program_authority_seed";
pub const LOCK_CP_AUTH_SEED: &str = "lock_cp_authority_seed";

pub const LOCKED_LIQUIDITY_SEED: &str = "locked_liquidity";
pub const LOCKED_POSITION_SEED: &str = "locked_position";

pub const LOCK_CLMM_POSITION_INS: [u8; 8] = [188, 37, 179, 131, 82, 150, 84, 73];
pub const COLLECT_CLMM_FEES_INS: [u8; 8] = [16, 72, 250, 198, 14, 162, 212, 19];
pub const LOCK_CP_LIQUIDITY_INS: [u8; 8] = [216, 157, 29, 78, 38, 51, 31, 26];
pub const COLLECT_CP_FEES_INS: [u8; 8] = [8, 30, 51, 199, 209, 184, 247, 133];

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct LockCpLiquidityInstruction {
    discriminator: [u8; 8],
    lp_amount: u64,
    with_metadata: bool,
}
impl LockCpLiquidityInstruction {
    #[inline]
    pub fn serialize_ix(lp_amount: u64, with_metadata: bool) -> Result<Vec<u8>> {
        Ok(LockCpLiquidityInstruction {
            discriminator: LOCK_CP_LIQUIDITY_INS,
            lp_amount,
            with_metadata,
        }
        .try_to_vec()
        .unwrap())
    }
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct CollectCpFeeInstruction {
    discriminator: [u8; 8],
    fee_lp_amount: u64,
}
impl CollectCpFeeInstruction {
    #[inline]
    pub fn serialize_ix(fee_lp_amount: u64) -> Result<Vec<u8>> {
        Ok(CollectCpFeeInstruction {
            discriminator: COLLECT_CP_FEES_INS,
            fee_lp_amount,
        }
        .try_to_vec()
        .unwrap())
    }
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct LockClmmPositionInstruction {
    discriminator: [u8; 8],
    with_metadata: bool,
}
impl LockClmmPositionInstruction {
    #[inline]
    pub fn serialize_ix(with_metadata: bool) -> Result<Vec<u8>> {
        Ok(LockClmmPositionInstruction {
            discriminator: LOCK_CLMM_POSITION_INS,
            with_metadata,
        }
        .try_to_vec()
        .unwrap())
    }
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct CollectClmmFeesInstruction {
    discriminator: [u8; 8],
}
impl CollectClmmFeesInstruction {
    #[inline]
    pub fn serialize_ix() -> Result<Vec<u8>> {
        Ok(CollectClmmFeesInstruction {
            discriminator: COLLECT_CLMM_FEES_INS,
        }
        .try_to_vec()
        .unwrap())
    }
}

#[derive(Default, Debug, BorshDeserialize)]
pub struct LockedCpLiquidityState {
    /// The Locked liquidity amount without claimed lp fee
    pub locked_lp_amount: u64,
    /// Claimed lp fee amount
    pub claimed_lp_amount: u64,
    /// Unclaimed lp fee amount
    pub unclaimed_lp_amount: u64,
    /// Last updated cp pool lp total supply
    pub last_lp: u64,
    /// Last updated cp pool k
    pub last_k: u128,
    /// Account update recent epoch
    pub recent_epoch: u64,
    /// The ID of the pool with which this record is connected
    pub pool_id: Pubkey,
    /// nft mint to check who has authority to collect fee
    pub fee_nft_mint: Pubkey,
    /// The owner who has locked liquidity
    pub locked_owner: Pubkey,
    /// The mint of locked lp token
    pub locked_lp_mint: Pubkey,
    /// Unused bytes for future upgrades.
    pub padding: [u64; 8],
}

impl LockedCpLiquidityState {
    pub const LEN: usize = 8 + 4 * 8 + 16 + 8 + 32 * 4 + 8 * 8;
    pub const DISCRIMINATOR: [u8; 8] = [25, 10, 238, 197, 207, 234, 73, 22];

    pub fn deserialize_account(account_data: &[u8]) -> Result<LockedCpLiquidityState> {
        if account_data.len() != Self::LEN {
            panic!("data len mismatch");
        }
        let discriminator = &account_data[..8];
        if discriminator != Self::DISCRIMINATOR.as_slice() {
            panic!("Discriminator mismatch");
        }
        let state = LockedCpLiquidityState::try_from_slice(&account_data[8..]).unwrap();
        Ok(state)
    }
}

#[derive(Default, Debug, BorshDeserialize)]
pub struct LockedClmmPositionState {
    /// Bump to identify PDA
    pub bump: [u8; 1],
    /// The owner who has locked clmm NFT
    pub position_owner: Pubkey,
    /// The ID of the pool with which this record is connected
    pub pool_id: Pubkey,
    /// The ID of the position with which this record is connected
    pub position_id: Pubkey,
    /// Program ATA locked NFT account or user ATA position NFT account
    pub locked_nft_account: Pubkey,
    /// nft mint to check who has authority to collect fee
    pub fee_nft_mint: Pubkey,
    /// account update recent epoch
    pub recent_epoch: u64,
    /// Unused bytes for future upgrades.
    pub padding: [u64; 8],
}

impl LockedClmmPositionState {
    pub const LEN: usize = 8 + 1 + 32 * 5 + 8 + 8 * 8;
    pub const DISCRIMINATOR: [u8; 8] = [52, 23, 5, 7, 170, 90, 108, 213];

    pub fn deserialize_account(account_data: &[u8]) -> Result<LockedClmmPositionState> {
        if account_data.len() != Self::LEN {
            panic!("data len mismatch");
        }
        let discriminator = &account_data[..8];
        if discriminator != Self::DISCRIMINATOR.as_slice() {
            panic!("Discriminator mismatch");
        }
        let state = LockedClmmPositionState::try_from_slice(&account_data[8..]).unwrap();
        Ok(state)
    }
}
