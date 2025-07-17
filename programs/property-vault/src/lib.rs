use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

declare_id!("PR0p3rtyV4u1t7h1s1sY0urPr0p3rtyT0k3n1z4t10n");

#[program]
pub mod property_vault {
    use super::*;

    /// Initialize a new property for tokenization
    pub fn initialize_property(
        ctx: Context<InitializeProperty>,
        property_id: String,
        total_tokens: u64,
        token_price: u64, // Price per token in USDC (6 decimals)
        expected_annual_yield: u16, // Basis points (e.g., 800 = 8%)
        property_details: PropertyDetails,
    ) -> Result<()> {
        let property = &mut ctx.accounts.property;
        
        property.property_id = property_id;
        property.authority = ctx.accounts.authority.key();
        property.mint = ctx.accounts.mint.key();
        property.total_tokens = total_tokens;
        property.tokens_sold = 0;
        property.token_price = token_price;
        property.expected_annual_yield = expected_annual_yield;
        property.total_yield_distributed = 0;
        property.status = PropertyStatus::Active;
        property.created_at = Clock::get()?.unix_timestamp;
        property.details = property_details;
        
        emit!(PropertyInitializedEvent {
            property_id: property.property_id.clone(),
            mint: property.mint,
            total_tokens,
            token_price,
            expected_yield: expected_annual_yield,
            timestamp: property.created_at,
        });

        Ok(())
    }

    /// Purchase property tokens
    pub fn purchase_property_tokens(
        ctx: Context<PurchasePropertyTokens>,
        amount: u64,
    ) -> Result<()> {
        let property = &mut ctx.accounts.property;
        let investor = &mut ctx.accounts.investor;
        
        require!(
            property.status == PropertyStatus::Active,
            ErrorCode::PropertyNotActive
        );
        
        require!(
            property.tokens_sold + amount <= property.total_tokens,
            ErrorCode::InsufficientTokensAvailable
        );
        
        let total_cost = amount * property.token_price;
        
        // Transfer USDC payment
        let cpi_accounts = Transfer {
            from: ctx.accounts.investor_usdc.to_account_info(),
            to: ctx.accounts.property_usdc.to_account_info(),
            authority: ctx.accounts.investor_authority.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, total_cost)?;
        
        // Mint property tokens to investor
        let seeds = &[
            b"property",
            property.property_id.as_bytes(),
            &[ctx.bumps.property],
        ];
        let signer = &[&seeds[..]];
        
        let cpi_accounts = token::MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.investor_token_account.to_account_info(),
            authority: ctx.accounts.property.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::mint_to(cpi_ctx, amount)?;
        
        // Update investor record
        if investor.property == Pubkey::default() {
            investor.investor = ctx.accounts.investor_authority.key();
            investor.property = property.key();
            investor.tokens_owned = amount;
            investor.total_invested = total_cost;
            investor.yield_claimed = 0;
            investor.first_purchase = Clock::get()?.unix_timestamp;
        } else {
            investor.tokens_owned += amount;
            investor.total_invested += total_cost;
        }
        
        property.tokens_sold += amount;
        investor.last_purchase = Clock::get()?.unix_timestamp;
        
        emit!(TokenPurchaseEvent {
            property_id: property.property_id.clone(),
            investor: ctx.accounts.investor_authority.key(),
            amount,
            price: property.token_price,
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    /// Distribute yield to token holders
    pub fn distribute_yield(
        ctx: Context<DistributeYield>,
        total_yield: u64,
        period_start: i64,
        period_end: i64,
    ) -> Result<()> {
        let property = &mut ctx.accounts.property;
        let yield_distribution = &mut ctx.accounts.yield_distribution;
        
        require!(
            ctx.accounts.authority.key() == property.authority,
            ErrorCode::Unauthorized
        );
        
        // Calculate yield per token
        let yield_per_token = total_yield / property.tokens_sold;
        
        yield_distribution.property = property.key();
        yield_distribution.total_yield = total_yield;
        yield_distribution.yield_per_token = yield_per_token;
        yield_distribution.period_start = period_start;
        yield_distribution.period_end = period_end;
        yield_distribution.distributed_at = Clock::get()?.unix_timestamp;
        yield_distribution.claimed_amount = 0;
        
        property.total_yield_distributed += total_yield;
        
        emit!(YieldDistributionEvent {
            property_id: property.property_id.clone(),
            total_yield,
            yield_per_token,
            period_start,
            period_end,
            timestamp: yield_distribution.distributed_at,
        });

        Ok(())
    }

    /// Claim yield for an investor
    pub fn claim_yield(
        ctx: Context<ClaimYield>,
        distribution_id: Pubkey,
    ) -> Result<()> {
        let investor = &mut ctx.accounts.investor;
        let yield_distribution = &ctx.accounts.yield_distribution;
        let yield_claim = &mut ctx.accounts.yield_claim;
        
        require!(
            yield_distribution.property == investor.property,
            ErrorCode::InvalidDistribution
        );
        
        // Check if already claimed
        if yield_claim.investor != Pubkey::default() {
            return Err(ErrorCode::AlreadyClaimed.into());
        }
        
        let claimable_amount = investor.tokens_owned * yield_distribution.yield_per_token;
        
        require!(claimable_amount > 0, ErrorCode::NoYieldToClaim);
        
        // Transfer USDC yield to investor
        let seeds = &[
            b"property",
            ctx.accounts.property.property_id.as_bytes(),
            &[ctx.bumps.property],
        ];
        let signer = &[&seeds[..]];
        
        let cpi_accounts = Transfer {
            from: ctx.accounts.property_usdc.to_account_info(),
            to: ctx.accounts.investor_usdc.to_account_info(),
            authority: ctx.accounts.property.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, claimable_amount)?;
        
        // Record the claim
        yield_claim.investor = investor.investor;
        yield_claim.distribution = distribution_id;
        yield_claim.amount = claimable_amount;
        yield_claim.claimed_at = Clock::get()?.unix_timestamp;
        
        investor.yield_claimed += claimable_amount;
        
        emit!(YieldClaimEvent {
            property_id: ctx.accounts.property.property_id.clone(),
            investor: investor.investor,
            amount: claimable_amount,
            distribution_id,
            timestamp: yield_claim.claimed_at,
        });

        Ok(())
    }

    /// Update property status
    pub fn update_property_status(
        ctx: Context<UpdatePropertyStatus>,
        new_status: PropertyStatus,
    ) -> Result<()> {
        let property = &mut ctx.accounts.property;
        
        require!(
            ctx.accounts.authority.key() == property.authority,
            ErrorCode::Unauthorized
        );
        
        let old_status = property.status;
        property.status = new_status;
        
        emit!(PropertyStatusUpdateEvent {
            property_id: property.property_id.clone(),
            old_status,
            new_status,
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(property_id: String)]
pub struct InitializeProperty<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + Property::LEN,
        seeds = [b"property", property_id.as_bytes()],
        bump
    )]
    pub property: Account<'info, Property>,
    
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct PurchasePropertyTokens<'info> {
    #[account(
        mut,
        seeds = [b"property", property.property_id.as_bytes()],
        bump
    )]
    pub property: Account<'info, Property>,
    
    #[account(
        init_if_needed,
        payer = investor_authority,
        space = 8 + Investor::LEN,
        seeds = [b"investor", property.key().as_ref(), investor_authority.key().as_ref()],
        bump
    )]
    pub investor: Account<'info, Investor>,
    
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    
    #[account(mut)]
    pub investor_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub investor_usdc: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub property_usdc: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub investor_authority: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DistributeYield<'info> {
    #[account(
        mut,
        seeds = [b"property", property.property_id.as_bytes()],
        bump
    )]
    pub property: Account<'info, Property>,
    
    #[account(
        init,
        payer = authority,
        space = 8 + YieldDistribution::LEN,
        seeds = [b"yield_distribution", property.key().as_ref(), &Clock::get()?.unix_timestamp.to_le_bytes()],
        bump
    )]
    pub yield_distribution: Account<'info, YieldDistribution>,
    
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ClaimYield<'info> {
    #[account(
        seeds = [b"property", property.property_id.as_bytes()],
        bump
    )]
    pub property: Account<'info, Property>,
    
    #[account(
        mut,
        seeds = [b"investor", property.key().as_ref(), investor_authority.key().as_ref()],
        bump
    )]
    pub investor: Account<'info, Investor>,
    
    #[account(
        seeds = [b"yield_distribution", property.key().as_ref(), &yield_distribution.distributed_at.to_le_bytes()],
        bump
    )]
    pub yield_distribution: Account<'info, YieldDistribution>,
    
    #[account(
        init,
        payer = investor_authority,
        space = 8 + YieldClaim::LEN,
        seeds = [b"yield_claim", yield_distribution.key().as_ref(), investor_authority.key().as_ref()],
        bump
    )]
    pub yield_claim: Account<'info, YieldClaim>,
    
    #[account(mut)]
    pub property_usdc: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub investor_usdc: Account<'info, TokenAccount>,
    
    pub investor_authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdatePropertyStatus<'info> {
    #[account(
        mut,
        seeds = [b"property", property.property_id.as_bytes()],
        bump
    )]
    pub property: Account<'info, Property>,
    
    pub authority: Signer<'info>,
}

#[account]
pub struct Property {
    pub property_id: String,
    pub authority: Pubkey,
    pub mint: Pubkey,
    pub total_tokens: u64,
    pub tokens_sold: u64,
    pub token_price: u64,
    pub expected_annual_yield: u16,
    pub total_yield_distributed: u64,
    pub status: PropertyStatus,
    pub created_at: i64,
    pub details: PropertyDetails,
}

impl Property {
    pub const LEN: usize = 4 + 32 + 32 + 32 + 8 + 8 + 8 + 2 + 8 + 1 + 8 + PropertyDetails::LEN;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct PropertyDetails {
    pub address: String,
    pub city: String,
    pub country: String,
    pub property_type: String, // "residential", "commercial", "mixed"
    pub square_footage: u32,
    pub bedrooms: u8,
    pub bathrooms: u8,
    pub year_built: u16,
    pub purchase_price: u64,
    pub estimated_value: u64,
    pub rental_income_monthly: u64,
}

impl PropertyDetails {
    pub const LEN: usize = 4 + 100 + 4 + 50 + 4 + 50 + 4 + 20 + 4 + 1 + 1 + 2 + 8 + 8 + 8;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum PropertyStatus {
    Active,
    SoldOut,
    Paused,
    Closed,
}

#[account]
pub struct Investor {
    pub investor: Pubkey,
    pub property: Pubkey,
    pub tokens_owned: u64,
    pub total_invested: u64,
    pub yield_claimed: u64,
    pub first_purchase: i64,
    pub last_purchase: i64,
}

impl Investor {
    pub const LEN: usize = 32 + 32 + 8 + 8 + 8 + 8 + 8;
}

#[account]
pub struct YieldDistribution {
    pub property: Pubkey,
    pub total_yield: u64,
    pub yield_per_token: u64,
    pub period_start: i64,
    pub period_end: i64,
    pub distributed_at: i64,
    pub claimed_amount: u64,
}

impl YieldDistribution {
    pub const LEN: usize = 32 + 8 + 8 + 8 + 8 + 8 + 8;
}

#[account]
pub struct YieldClaim {
    pub investor: Pubkey,
    pub distribution: Pubkey,
    pub amount: u64,
    pub claimed_at: i64,
}

impl YieldClaim {
    pub const LEN: usize = 32 + 32 + 8 + 8;
}

#[event]
pub struct PropertyInitializedEvent {
    pub property_id: String,
    pub mint: Pubkey,
    pub total_tokens: u64,
    pub token_price: u64,
    pub expected_yield: u16,
    pub timestamp: i64,
}

#[event]
pub struct TokenPurchaseEvent {
    pub property_id: String,
    pub investor: Pubkey,
    pub amount: u64,
    pub price: u64,
    pub timestamp: i64,
}

#[event]
pub struct YieldDistributionEvent {
    pub property_id: String,
    pub total_yield: u64,
    pub yield_per_token: u64,
    pub period_start: i64,
    pub period_end: i64,
    pub timestamp: i64,
}

#[event]
pub struct YieldClaimEvent {
    pub property_id: String,
    pub investor: Pubkey,
    pub amount: u64,
    pub distribution_id: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct PropertyStatusUpdateEvent {
    pub property_id: String,
    pub old_status: PropertyStatus,
    pub new_status: PropertyStatus,
    pub timestamp: i64,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Property is not active")]
    PropertyNotActive,
    #[msg("Insufficient tokens available")]
    InsufficientTokensAvailable,
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Invalid distribution")]
    InvalidDistribution,
    #[msg("Already claimed")]
    AlreadyClaimed,
    #[msg("No yield to claim")]
    NoYieldToClaim,
}
