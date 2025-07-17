use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

declare_id!("Y13ldD1str1but0rR3nt4lR3v3nu3Sh4r1ngSm4rtC0ntr4ct");

#[program]
pub mod yield_distributor {
    use super::*;

    /// Initialize yield distribution pool
    pub fn initialize_yield_pool(
        ctx: Context<InitializeYieldPool>,
        pool_id: String,
        property_mint: Pubkey,
        distribution_frequency: DistributionFrequency,
    ) -> Result<()> {
        let yield_pool = &mut ctx.accounts.yield_pool;
        
        yield_pool.pool_id = pool_id;
        yield_pool.authority = ctx.accounts.authority.key();
        yield_pool.property_mint = property_mint;
        yield_pool.usdc_vault = ctx.accounts.usdc_vault.key();
        yield_pool.total_deposited = 0;
        yield_pool.total_distributed = 0;
        yield_pool.distribution_frequency = distribution_frequency;
        yield_pool.last_distribution = 0;
        yield_pool.distributions_count = 0;
        yield_pool.status = PoolStatus::Active;
        yield_pool.created_at = Clock::get()?.unix_timestamp;
        
        emit!(YieldPoolCreated {
            pool_id: yield_pool.pool_id.clone(),
            property_mint,
            usdc_vault: yield_pool.usdc_vault,
            distribution_frequency: distribution_frequency.clone(),
            timestamp: yield_pool.created_at,
        });

        Ok(())
    }

    /// Deposit yield to the pool
    pub fn deposit_yield(
        ctx: Context<DepositYield>,
        amount: u64,
        yield_period: String,
    ) -> Result<()> {
        let yield_pool = &mut ctx.accounts.yield_pool;
        
        require!(
            yield_pool.status == PoolStatus::Active,
            ErrorCode::PoolNotActive
        );
        
        // Transfer USDC to the yield vault
        let cpi_accounts = Transfer {
            from: ctx.accounts.depositor_usdc.to_account_info(),
            to: ctx.accounts.usdc_vault.to_account_info(),
            authority: ctx.accounts.depositor.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;
        
        yield_pool.total_deposited += amount;
        
        emit!(YieldDeposited {
            pool_id: yield_pool.pool_id.clone(),
            depositor: ctx.accounts.depositor.key(),
            amount,
            yield_period,
            total_deposited: yield_pool.total_deposited,
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    /// Create distribution snapshot
    pub fn create_distribution_snapshot(
        ctx: Context<CreateDistributionSnapshot>,
        snapshot_id: String,
        total_tokens_eligible: u64,
        yield_amount: u64,
    ) -> Result<()> {
        let yield_pool = &ctx.accounts.yield_pool;
        let distribution = &mut ctx.accounts.distribution;
        
        require!(
            ctx.accounts.authority.key() == yield_pool.authority,
            ErrorCode::Unauthorized
        );
        
        require!(
            yield_amount <= yield_pool.total_deposited - yield_pool.total_distributed,
            ErrorCode::InsufficientFunds
        );
        
        distribution.snapshot_id = snapshot_id;
        distribution.yield_pool = yield_pool.key();
        distribution.total_tokens_eligible = total_tokens_eligible;
        distribution.yield_amount = yield_amount;
        distribution.distributed_amount = 0;
        distribution.claims_count = 0;
        distribution.status = DistributionStatus::Active;
        distribution.created_at = Clock::get()?.unix_timestamp;
        distribution.expires_at = distribution.created_at + 30 * 24 * 60 * 60; // 30 days expiry
        
        emit!(DistributionCreated {
            snapshot_id: distribution.snapshot_id.clone(),
            pool_id: yield_pool.pool_id.clone(),
            yield_amount,
            total_tokens_eligible,
            expires_at: distribution.expires_at,
            timestamp: distribution.created_at,
        });

        Ok(())
    }

    /// Claim yield for token holder
    pub fn claim_yield(
        ctx: Context<ClaimYield>,
        token_balance: u64,
        merkle_proof: Vec<[u8; 32]>,
    ) -> Result<()> {
        let yield_pool = &mut ctx.accounts.yield_pool;
        let distribution = &mut ctx.accounts.distribution;
        let claim_record = &mut ctx.accounts.claim_record;
        
        require!(
            distribution.status == DistributionStatus::Active,
            ErrorCode::DistributionNotActive
        );
        
        require!(
            Clock::get()?.unix_timestamp <= distribution.expires_at,
            ErrorCode::DistributionExpired
        );
        
        require!(
            !claim_record.has_claimed,
            ErrorCode::AlreadyClaimed
        );
        
        // Verify merkle proof (simplified - in production use a proper merkle tree library)
        require!(
            verify_merkle_proof(&merkle_proof, ctx.accounts.claimer.key(), token_balance),
            ErrorCode::InvalidProof
        );
        
        // Calculate yield amount based on token balance
        let yield_amount = (distribution.yield_amount * token_balance) / distribution.total_tokens_eligible;
        
        require!(
            yield_amount > 0,
            ErrorCode::NoYieldToClaim
        );
        
        // Transfer yield to claimer
        let seeds = &[
            b"yield_pool",
            yield_pool.pool_id.as_bytes(),
            &[ctx.bumps.yield_pool],
        ];
        let signer = &[&seeds[..]];
        
        let cpi_accounts = Transfer {
            from: ctx.accounts.usdc_vault.to_account_info(),
            to: ctx.accounts.claimer_usdc.to_account_info(),
            authority: ctx.accounts.yield_pool.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, yield_amount)?;
        
        // Update records
        claim_record.has_claimed = true;
        claim_record.claimed_amount = yield_amount;
        claim_record.claimed_at = Clock::get()?.unix_timestamp;
        
        distribution.distributed_amount += yield_amount;
        distribution.claims_count += 1;
        
        yield_pool.total_distributed += yield_amount;
        
        emit!(YieldClaimed {
            claimer: ctx.accounts.claimer.key(),
            distribution_id: distribution.snapshot_id.clone(),
            token_balance,
            yield_amount,
            timestamp: claim_record.claimed_at,
        });

        Ok(())
    }

    /// Batch process yield claims (for automated distributions)
    pub fn batch_process_claims(
        ctx: Context<BatchProcessClaims>,
        claim_data: Vec<ClaimData>,
    ) -> Result<()> {
        let yield_pool = &mut ctx.accounts.yield_pool;
        let distribution = &mut ctx.accounts.distribution;
        
        require!(
            ctx.accounts.authority.key() == yield_pool.authority,
            ErrorCode::Unauthorized
        );
        
        require!(
            distribution.status == DistributionStatus::Active,
            ErrorCode::DistributionNotActive
        );
        
        let mut total_processed = 0u64;
        let mut claims_processed = 0u32;
        
        for claim in claim_data.iter() {
            // In a real implementation, you would iterate through remaining accounts
            // and process each claim individually
            total_processed += claim.yield_amount;
            claims_processed += 1;
        }
        
        distribution.distributed_amount += total_processed;
        distribution.claims_count += claims_processed;
        yield_pool.total_distributed += total_processed;
        
        emit!(BatchClaimsProcessed {
            distribution_id: distribution.snapshot_id.clone(),
            claims_processed,
            total_amount: total_processed,
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    /// Finalize distribution (after expiry or completion)
    pub fn finalize_distribution(ctx: Context<FinalizeDistribution>) -> Result<()> {
        let yield_pool = &ctx.accounts.yield_pool;
        let distribution = &mut ctx.accounts.distribution;
        
        require!(
            ctx.accounts.authority.key() == yield_pool.authority,
            ErrorCode::Unauthorized
        );
        
        require!(
            Clock::get()?.unix_timestamp > distribution.expires_at ||
            distribution.distributed_amount == distribution.yield_amount,
            ErrorCode::CannotFinalize
        );
        
        distribution.status = DistributionStatus::Finalized;
        
        emit!(DistributionFinalized {
            distribution_id: distribution.snapshot_id.clone(),
            total_distributed: distribution.distributed_amount,
            unclaimed_amount: distribution.yield_amount - distribution.distributed_amount,
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    /// Emergency pause pool
    pub fn pause_pool(ctx: Context<PausePool>) -> Result<()> {
        let yield_pool = &mut ctx.accounts.yield_pool;
        
        require!(
            ctx.accounts.authority.key() == yield_pool.authority,
            ErrorCode::Unauthorized
        );
        
        yield_pool.status = PoolStatus::Paused;
        
        emit!(PoolPaused {
            pool_id: yield_pool.pool_id.clone(),
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    /// Resume paused pool
    pub fn resume_pool(ctx: Context<ResumePool>) -> Result<()> {
        let yield_pool = &mut ctx.accounts.yield_pool;
        
        require!(
            ctx.accounts.authority.key() == yield_pool.authority,
            ErrorCode::Unauthorized
        );
        
        yield_pool.status = PoolStatus::Active;
        
        emit!(PoolResumed {
            pool_id: yield_pool.pool_id.clone(),
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }
}

// Helper function for merkle proof verification (simplified)
fn verify_merkle_proof(
    proof: &[[u8; 32]],
    claimer: Pubkey,
    token_balance: u64,
) -> bool {
    // In production, implement proper merkle tree verification
    // This is a simplified version for demonstration
    proof.len() > 0 && token_balance > 0
}

#[derive(Accounts)]
#[instruction(pool_id: String)]
pub struct InitializeYieldPool<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + YieldPool::LEN,
        seeds = [b"yield_pool", pool_id.as_bytes()],
        bump
    )]
    pub yield_pool: Account<'info, YieldPool>,
    
    pub usdc_vault: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DepositYield<'info> {
    #[account(mut)]
    pub yield_pool: Account<'info, YieldPool>,
    
    #[account(mut)]
    pub depositor_usdc: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub usdc_vault: Account<'info, TokenAccount>,
    
    pub depositor: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(snapshot_id: String)]
pub struct CreateDistributionSnapshot<'info> {
    pub yield_pool: Account<'info, YieldPool>,
    
    #[account(
        init,
        payer = authority,
        space = 8 + Distribution::LEN,
        seeds = [b"distribution", snapshot_id.as_bytes()],
        bump
    )]
    pub distribution: Account<'info, Distribution>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ClaimYield<'info> {
    #[account(
        mut,
        seeds = [b"yield_pool", yield_pool.pool_id.as_bytes()],
        bump
    )]
    pub yield_pool: Account<'info, YieldPool>,
    
    #[account(mut)]
    pub distribution: Account<'info, Distribution>,
    
    #[account(
        init_if_needed,
        payer = claimer,
        space = 8 + ClaimRecord::LEN,
        seeds = [b"claim_record", distribution.key().as_ref(), claimer.key().as_ref()],
        bump
    )]
    pub claim_record: Account<'info, ClaimRecord>,
    
    #[account(mut)]
    pub usdc_vault: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub claimer_usdc: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub claimer: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct BatchProcessClaims<'info> {
    #[account(mut)]
    pub yield_pool: Account<'info, YieldPool>,
    
    #[account(mut)]
    pub distribution: Account<'info, Distribution>,
    
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct FinalizeDistribution<'info> {
    pub yield_pool: Account<'info, YieldPool>,
    
    #[account(mut)]
    pub distribution: Account<'info, Distribution>,
    
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct PausePool<'info> {
    #[account(mut)]
    pub yield_pool: Account<'info, YieldPool>,
    
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct ResumePool<'info> {
    #[account(mut)]
    pub yield_pool: Account<'info, YieldPool>,
    
    pub authority: Signer<'info>,
}

#[account]
pub struct YieldPool {
    pub pool_id: String,
    pub authority: Pubkey,
    pub property_mint: Pubkey,
    pub usdc_vault: Pubkey,
    pub total_deposited: u64,
    pub total_distributed: u64,
    pub distribution_frequency: DistributionFrequency,
    pub last_distribution: i64,
    pub distributions_count: u32,
    pub status: PoolStatus,
    pub created_at: i64,
}

impl YieldPool {
    pub const LEN: usize = 4 + 50 + 32 + 32 + 32 + 8 + 8 + 1 + 8 + 4 + 1 + 8;
}

#[account]
pub struct Distribution {
    pub snapshot_id: String,
    pub yield_pool: Pubkey,
    pub total_tokens_eligible: u64,
    pub yield_amount: u64,
    pub distributed_amount: u64,
    pub claims_count: u32,
    pub status: DistributionStatus,
    pub created_at: i64,
    pub expires_at: i64,
}

impl Distribution {
    pub const LEN: usize = 4 + 50 + 32 + 8 + 8 + 8 + 4 + 1 + 8 + 8;
}

#[account]
pub struct ClaimRecord {
    pub claimer: Pubkey,
    pub distribution: Pubkey,
    pub has_claimed: bool,
    pub claimed_amount: u64,
    pub claimed_at: i64,
}

impl ClaimRecord {
    pub const LEN: usize = 32 + 32 + 1 + 8 + 8;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum DistributionFrequency {
    Monthly,
    Quarterly,
    SemiAnnually,
    Annually,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum PoolStatus {
    Active,
    Paused,
    Closed,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum DistributionStatus {
    Active,
    Finalized,
    Expired,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ClaimData {
    pub claimer: Pubkey,
    pub token_balance: u64,
    pub yield_amount: u64,
}

#[event]
pub struct YieldPoolCreated {
    pub pool_id: String,
    pub property_mint: Pubkey,
    pub usdc_vault: Pubkey,
    pub distribution_frequency: DistributionFrequency,
    pub timestamp: i64,
}

#[event]
pub struct YieldDeposited {
    pub pool_id: String,
    pub depositor: Pubkey,
    pub amount: u64,
    pub yield_period: String,
    pub total_deposited: u64,
    pub timestamp: i64,
}

#[event]
pub struct DistributionCreated {
    pub snapshot_id: String,
    pub pool_id: String,
    pub yield_amount: u64,
    pub total_tokens_eligible: u64,
    pub expires_at: i64,
    pub timestamp: i64,
}

#[event]
pub struct YieldClaimed {
    pub claimer: Pubkey,
    pub distribution_id: String,
    pub token_balance: u64,
    pub yield_amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct BatchClaimsProcessed {
    pub distribution_id: String,
    pub claims_processed: u32,
    pub total_amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct DistributionFinalized {
    pub distribution_id: String,
    pub total_distributed: u64,
    pub unclaimed_amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct PoolPaused {
    pub pool_id: String,
    pub timestamp: i64,
}

#[event]
pub struct PoolResumed {
    pub pool_id: String,
    pub timestamp: i64,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Pool is not active")]
    PoolNotActive,
    #[msg("Unauthorized access")]
    Unauthorized,
    #[msg("Insufficient funds in the pool")]
    InsufficientFunds,
    #[msg("Distribution is not active")]
    DistributionNotActive,
    #[msg("Distribution has expired")]
    DistributionExpired,
    #[msg("User has already claimed for this distribution")]
    AlreadyClaimed,
    #[msg("Invalid merkle proof")]
    InvalidProof,
    #[msg("No yield to claim")]
    NoYieldToClaim,
    #[msg("Cannot finalize distribution yet")]
    CannotFinalize,
}
