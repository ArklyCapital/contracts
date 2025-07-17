use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

declare_id!("ARKLyT0k3nM1nt7h1s1sY0urT0k3nPr0gr4mId3nt1f13r");

#[program]
pub mod arkly_token {
    use super::*;

    /// Initialize the ARKLY token with total supply and tokenomics
    pub fn initialize_token(
        ctx: Context<InitializeToken>,
        total_supply: u64,
        decimals: u8,
    ) -> Result<()> {
        let token_info = &mut ctx.accounts.token_info;
        token_info.total_supply = total_supply;
        token_info.circulating_supply = 0;
        token_info.decimals = decimals;
        token_info.authority = ctx.accounts.authority.key();
        token_info.mint = ctx.accounts.mint.key();
        
        // Initialize tokenomics allocations
        token_info.allocations = TokenomicsAllocations {
            seed_round: AllocationInfo {
                amount: total_supply * 12 / 100, // 12%
                price: 83_000_000, // $0.083 in micro-dollars
                cliff_months: 6,
                vesting_months: 12,
                released: 0,
            },
            public_presale: AllocationInfo {
                amount: total_supply * 75 / 1000, // 7.5%
                price: 100_000_000, // $0.10 in micro-dollars
                cliff_months: 0,
                vesting_months: 0, // 100% at TGE
                released: 0,
            },
            liquidity_pool: AllocationInfo {
                amount: total_supply * 10 / 100, // 10%
                price: 150_000_000, // $0.15 listing price
                cliff_months: 0,
                vesting_months: 1, // 50% at TGE, 50% after 30 days
                released: 0,
            },
            team_advisors: AllocationInfo {
                amount: total_supply * 15 / 100, // 15%
                price: 0,
                cliff_months: 12,
                vesting_months: 24,
                released: 0,
            },
            ecosystem_rewards: AllocationInfo {
                amount: total_supply * 25 / 100, // 25%
                price: 0,
                cliff_months: 0,
                vesting_months: 36,
                released: 0,
            },
            treasury_dev: AllocationInfo {
                amount: total_supply * 20 / 100, // 20%
                price: 0,
                cliff_months: 0,
                vesting_months: 0, // Roadmap-based
                released: 0,
            },
            strategic_partners: AllocationInfo {
                amount: total_supply * 5 / 100, // 5%
                price: 0,
                cliff_months: 6,
                vesting_months: 12,
                released: 0,
            },
            community_airdrops: AllocationInfo {
                amount: total_supply * 8 / 100, // 8%
                price: 0,
                cliff_months: 0,
                vesting_months: 0, // Claim-based
                released: 0,
            },
        };

        Ok(())
    }

    /// Purchase tokens during presale
    pub fn purchase_presale(
        ctx: Context<PurchasePresale>,
        amount: u64,
        allocation_type: u8,
    ) -> Result<()> {
        let token_info = &mut ctx.accounts.token_info;
        let user_purchase = &mut ctx.accounts.user_purchase;
        
        // Validate allocation type and availability
        let allocation = match allocation_type {
            0 => &mut token_info.allocations.seed_round,
            1 => &mut token_info.allocations.public_presale,
            _ => return Err(ErrorCode::InvalidAllocationType.into()),
        };
        
        require!(
            allocation.released + amount <= allocation.amount,
            ErrorCode::InsufficientAllocation
        );
        
        // Calculate payment required
        let payment_required = amount * allocation.price / 1_000_000_000; // Convert from micro-dollars
        
        // Transfer USDC payment
        let cpi_accounts = Transfer {
            from: ctx.accounts.user_usdc.to_account_info(),
            to: ctx.accounts.treasury_usdc.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, payment_required)?;
        
        // Update purchase record
        user_purchase.user = ctx.accounts.user.key();
        user_purchase.allocation_type = allocation_type;
        user_purchase.amount_purchased += amount;
        user_purchase.total_paid += payment_required;
        user_purchase.last_purchase = Clock::get()?.unix_timestamp;
        
        // Update allocation
        allocation.released += amount;
        token_info.circulating_supply += amount;
        
        emit!(TokenPurchaseEvent {
            user: ctx.accounts.user.key(),
            amount,
            price: allocation.price,
            allocation_type,
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    /// Claim vested tokens
    pub fn claim_vested_tokens(ctx: Context<ClaimVestedTokens>) -> Result<()> {
        let user_purchase = &mut ctx.accounts.user_purchase;
        let token_info = &ctx.accounts.token_info;
        
        let claimable_amount = calculate_vested_amount(
            user_purchase,
            token_info,
            Clock::get()?.unix_timestamp,
        )?;
        
        require!(claimable_amount > 0, ErrorCode::NoTokensToCllaim);
        
        // Mint tokens to user
        let seeds = &[
            b"token_info",
            &[ctx.bumps.token_info],
        ];
        let signer = &[&seeds[..]];
        
        let cpi_accounts = token::MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.user_token_account.to_account_info(),
            authority: ctx.accounts.token_info.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::mint_to(cpi_ctx, claimable_amount)?;
        
        user_purchase.amount_claimed += claimable_amount;
        user_purchase.last_claim = Clock::get()?.unix_timestamp;
        
        emit!(TokenClaimEvent {
            user: ctx.accounts.user.key(),
            amount: claimable_amount,
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeToken<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + TokenInfo::LEN,
        seeds = [b"token_info"],
        bump
    )]
    pub token_info: Account<'info, TokenInfo>,
    
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct PurchasePresale<'info> {
    #[account(
        mut,
        seeds = [b"token_info"],
        bump
    )]
    pub token_info: Account<'info, TokenInfo>,
    
    #[account(
        init_if_needed,
        payer = user,
        space = 8 + UserPurchase::LEN,
        seeds = [b"user_purchase", user.key().as_ref()],
        bump
    )]
    pub user_purchase: Account<'info, UserPurchase>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(mut)]
    pub user_usdc: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub treasury_usdc: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ClaimVestedTokens<'info> {
    #[account(
        seeds = [b"token_info"],
        bump
    )]
    pub token_info: Account<'info, TokenInfo>,
    
    #[account(
        mut,
        seeds = [b"user_purchase", user.key().as_ref()],
        bump
    )]
    pub user_purchase: Account<'info, UserPurchase>,
    
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    
    pub user: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[account]
pub struct TokenInfo {
    pub total_supply: u64,
    pub circulating_supply: u64,
    pub decimals: u8,
    pub authority: Pubkey,
    pub mint: Pubkey,
    pub allocations: TokenomicsAllocations,
}

impl TokenInfo {
    pub const LEN: usize = 8 + 8 + 1 + 32 + 32 + TokenomicsAllocations::LEN;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct TokenomicsAllocations {
    pub seed_round: AllocationInfo,
    pub public_presale: AllocationInfo,
    pub liquidity_pool: AllocationInfo,
    pub team_advisors: AllocationInfo,
    pub ecosystem_rewards: AllocationInfo,
    pub treasury_dev: AllocationInfo,
    pub strategic_partners: AllocationInfo,
    pub community_airdrops: AllocationInfo,
}

impl TokenomicsAllocations {
    pub const LEN: usize = AllocationInfo::LEN * 8;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct AllocationInfo {
    pub amount: u64,
    pub price: u64, // In micro-dollars
    pub cliff_months: u8,
    pub vesting_months: u8,
    pub released: u64,
}

impl AllocationInfo {
    pub const LEN: usize = 8 + 8 + 1 + 1 + 8;
}

#[account]
pub struct UserPurchase {
    pub user: Pubkey,
    pub allocation_type: u8,
    pub amount_purchased: u64,
    pub amount_claimed: u64,
    pub total_paid: u64,
    pub purchase_timestamp: i64,
    pub last_purchase: i64,
    pub last_claim: i64,
}

impl UserPurchase {
    pub const LEN: usize = 32 + 1 + 8 + 8 + 8 + 8 + 8 + 8;
}

#[event]
pub struct TokenPurchaseEvent {
    pub user: Pubkey,
    pub amount: u64,
    pub price: u64,
    pub allocation_type: u8,
    pub timestamp: i64,
}

#[event]
pub struct TokenClaimEvent {
    pub user: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid allocation type")]
    InvalidAllocationType,
    #[msg("Insufficient allocation remaining")]
    InsufficientAllocation,
    #[msg("No tokens available to claim")]
    NoTokensToCllaim,
    #[msg("Cliff period not reached")]
    CliffNotReached,
    #[msg("Unauthorized")]
    Unauthorized,
}

/// Calculate vested amount based on cliff and vesting schedule
fn calculate_vested_amount(
    user_purchase: &UserPurchase,
    token_info: &TokenInfo,
    current_timestamp: i64,
) -> Result<u64> {
    let allocation = match user_purchase.allocation_type {
        0 => &token_info.allocations.seed_round,
        1 => &token_info.allocations.public_presale,
        2 => &token_info.allocations.liquidity_pool,
        3 => &token_info.allocations.team_advisors,
        4 => &token_info.allocations.ecosystem_rewards,
        5 => &token_info.allocations.treasury_dev,
        6 => &token_info.allocations.strategic_partners,
        7 => &token_info.allocations.community_airdrops,
        _ => return Err(ErrorCode::InvalidAllocationType.into()),
    };
    
    let months_since_purchase = (current_timestamp - user_purchase.purchase_timestamp) / (30 * 24 * 60 * 60);
    
    // Check cliff period
    if months_since_purchase < allocation.cliff_months as i64 {
        return Ok(0);
    }
    
    // Calculate vested amount
    let vested_amount = if allocation.vesting_months == 0 {
        // Immediate vesting (like public presale)
        user_purchase.amount_purchased
    } else {
        let vesting_months_passed = (months_since_purchase - allocation.cliff_months as i64)
            .min(allocation.vesting_months as i64);
        
        user_purchase.amount_purchased * vesting_months_passed as u64 / allocation.vesting_months as u64
    };
    
    Ok(vested_amount.saturating_sub(user_purchase.amount_claimed))
}
