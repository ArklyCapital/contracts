use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

declare_id!("G0v3rn4nc3V0t1ngD4oM4n4g3m3ntSm4rtC0ntr4ct1d");

#[program]
pub mod governance {
    use super::*;

    /// Initialize governance system
    pub fn initialize_governance(
        ctx: Context<InitializeGovernance>,
        min_proposal_stake: u64,
        voting_period: i64,
        execution_delay: i64,
    ) -> Result<()> {
        let governance = &mut ctx.accounts.governance;
        
        governance.authority = ctx.accounts.authority.key();
        governance.arkly_mint = ctx.accounts.arkly_mint.key();
        governance.min_proposal_stake = min_proposal_stake;
        governance.voting_period = voting_period;
        governance.execution_delay = execution_delay;
        governance.proposal_count = 0;
        governance.total_staked = 0;
        
        Ok(())
    }

    /// Create a new governance proposal
    pub fn create_proposal(
        ctx: Context<CreateProposal>,
        title: String,
        description: String,
        proposal_type: ProposalType,
        execution_data: Vec<u8>,
    ) -> Result<()> {
        let governance = &mut ctx.accounts.governance;
        let proposal = &mut ctx.accounts.proposal;
        
        // Check if proposer has enough staked tokens
        require!(
            ctx.accounts.proposer_stake.amount >= governance.min_proposal_stake,
            ErrorCode::InsufficientStake
        );
        
        proposal.id = governance.proposal_count;
        proposal.proposer = ctx.accounts.proposer.key();
        proposal.title = title;
        proposal.description = description;
        proposal.proposal_type = proposal_type;
        proposal.execution_data = execution_data;
        proposal.votes_for = 0;
        proposal.votes_against = 0;
        proposal.status = ProposalStatus::Active;
        proposal.created_at = Clock::get()?.unix_timestamp;
        proposal.voting_ends_at = proposal.created_at + governance.voting_period;
        proposal.execution_eta = 0;
        
        governance.proposal_count += 1;
        
        emit!(ProposalCreated {
            proposal_id: proposal.id,
            proposer: proposal.proposer,
            title: proposal.title.clone(),
            proposal_type: proposal.proposal_type.clone(),
            voting_ends_at: proposal.voting_ends_at,
        });

        Ok(())
    }

    /// Vote on a proposal
    pub fn vote(
        ctx: Context<Vote>,
        support: bool,
    ) -> Result<()> {
        let proposal = &mut ctx.accounts.proposal;
        let voter_record = &mut ctx.accounts.voter_record;
        
        require!(
            Clock::get()?.unix_timestamp <= proposal.voting_ends_at,
            ErrorCode::VotingPeriodEnded
        );
        
        require!(
            proposal.status == ProposalStatus::Active,
            ErrorCode::ProposalNotActive
        );
        
        require!(
            voter_record.has_voted == false,
            ErrorCode::AlreadyVoted
        );
        
        let voting_power = ctx.accounts.voter_token_account.amount;
        
        if support {
            proposal.votes_for += voting_power;
        } else {
            proposal.votes_against += voting_power;
        }
        
        voter_record.has_voted = true;
        voter_record.support = support;
        voter_record.voting_power = voting_power;
        voter_record.voted_at = Clock::get()?.unix_timestamp;
        
        emit!(VoteCast {
            proposal_id: proposal.id,
            voter: ctx.accounts.voter.key(),
            support,
            voting_power,
            timestamp: voter_record.voted_at,
        });

        Ok(())
    }

    /// Queue proposal for execution
    pub fn queue_proposal(ctx: Context<QueueProposal>) -> Result<()> {
        let governance = &ctx.accounts.governance;
        let proposal = &mut ctx.accounts.proposal;
        
        require!(
            Clock::get()?.unix_timestamp > proposal.voting_ends_at,
            ErrorCode::VotingPeriodNotEnded
        );
        
        require!(
            proposal.status == ProposalStatus::Active,
            ErrorCode::ProposalNotActive
        );
        
        // Check if proposal passed (simple majority)
        if proposal.votes_for > proposal.votes_against {
            proposal.status = ProposalStatus::Queued;
            proposal.execution_eta = Clock::get()?.unix_timestamp + governance.execution_delay;
            
            emit!(ProposalQueued {
                proposal_id: proposal.id,
                execution_eta: proposal.execution_eta,
            });
        } else {
            proposal.status = ProposalStatus::Defeated;
            
            emit!(ProposalDefeated {
                proposal_id: proposal.id,
            });
        }

        Ok(())
    }

    /// Execute a queued proposal
    pub fn execute_proposal(ctx: Context<ExecuteProposal>) -> Result<()> {
        let proposal = &mut ctx.accounts.proposal;
        
        require!(
            proposal.status == ProposalStatus::Queued,
            ErrorCode::ProposalNotQueued
        );
        
        require!(
            Clock::get()?.unix_timestamp >= proposal.execution_eta,
            ErrorCode::ExecutionDelayNotPassed
        );
        
        // Execute based on proposal type
        match proposal.proposal_type {
            ProposalType::ParameterChange => {
                // Execute parameter change logic
                msg!("Executing parameter change proposal");
            }
            ProposalType::TreasurySpend => {
                // Execute treasury spending logic
                msg!("Executing treasury spending proposal");
            }
            ProposalType::ProtocolUpgrade => {
                // Execute protocol upgrade logic
                msg!("Executing protocol upgrade proposal");
            }
            ProposalType::PropertyListing => {
                // Execute property listing logic
                msg!("Executing property listing proposal");
            }
        }
        
        proposal.status = ProposalStatus::Executed;
        
        emit!(ProposalExecuted {
            proposal_id: proposal.id,
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    /// Stake ARKLY tokens for governance participation
    pub fn stake_tokens(ctx: Context<StakeTokens>, amount: u64) -> Result<()> {
        let governance = &mut ctx.accounts.governance;
        let stake_account = &mut ctx.accounts.stake_account;
        
        // Transfer tokens to governance vault
        let cpi_accounts = anchor_spl::token::Transfer {
            from: ctx.accounts.user_token_account.to_account_info(),
            to: ctx.accounts.governance_vault.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        anchor_spl::token::transfer(cpi_ctx, amount)?;
        
        stake_account.staked_amount += amount;
        stake_account.last_stake_time = Clock::get()?.unix_timestamp;
        governance.total_staked += amount;
        
        emit!(TokensStaked {
            user: ctx.accounts.user.key(),
            amount,
            total_staked: stake_account.staked_amount,
            timestamp: stake_account.last_stake_time,
        });

        Ok(())
    }

    /// Unstake ARKLY tokens
    pub fn unstake_tokens(ctx: Context<UnstakeTokens>, amount: u64) -> Result<()> {
        let governance = &mut ctx.accounts.governance;
        let stake_account = &mut ctx.accounts.stake_account;
        
        require!(
            stake_account.staked_amount >= amount,
            ErrorCode::InsufficientStakedAmount
        );
        
        // Transfer tokens back to user
        let seeds = &[b"governance", &[ctx.bumps.governance]];
        let signer = &[&seeds[..]];
        
        let cpi_accounts = anchor_spl::token::Transfer {
            from: ctx.accounts.governance_vault.to_account_info(),
            to: ctx.accounts.user_token_account.to_account_info(),
            authority: ctx.accounts.governance.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        anchor_spl::token::transfer(cpi_ctx, amount)?;
        
        stake_account.staked_amount -= amount;
        governance.total_staked -= amount;
        
        emit!(TokensUnstaked {
            user: ctx.accounts.user.key(),
            amount,
            remaining_staked: stake_account.staked_amount,
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeGovernance<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + Governance::LEN,
        seeds = [b"governance"],
        bump
    )]
    pub governance: Account<'info, Governance>,
    
    pub arkly_mint: Account<'info, anchor_spl::token::Mint>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(title: String)]
pub struct CreateProposal<'info> {
    #[account(mut)]
    pub governance: Account<'info, Governance>,
    
    #[account(
        init,
        payer = proposer,
        space = 8 + Proposal::LEN,
        seeds = [b"proposal", governance.proposal_count.to_le_bytes().as_ref()],
        bump
    )]
    pub proposal: Account<'info, Proposal>,
    
    pub proposer_stake: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub proposer: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Vote<'info> {
    #[account(mut)]
    pub proposal: Account<'info, Proposal>,
    
    #[account(
        init_if_needed,
        payer = voter,
        space = 8 + VoterRecord::LEN,
        seeds = [b"voter_record", proposal.key().as_ref(), voter.key().as_ref()],
        bump
    )]
    pub voter_record: Account<'info, VoterRecord>,
    
    pub voter_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub voter: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct QueueProposal<'info> {
    pub governance: Account<'info, Governance>,
    
    #[account(mut)]
    pub proposal: Account<'info, Proposal>,
}

#[derive(Accounts)]
pub struct ExecuteProposal<'info> {
    #[account(mut)]
    pub proposal: Account<'info, Proposal>,
}

#[derive(Accounts)]
pub struct StakeTokens<'info> {
    #[account(mut)]
    pub governance: Account<'info, Governance>,
    
    #[account(
        init_if_needed,
        payer = user,
        space = 8 + StakeAccount::LEN,
        seeds = [b"stake", user.key().as_ref()],
        bump
    )]
    pub stake_account: Account<'info, StakeAccount>,
    
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub governance_vault: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UnstakeTokens<'info> {
    #[account(
        mut,
        seeds = [b"governance"],
        bump
    )]
    pub governance: Account<'info, Governance>,
    
    #[account(mut)]
    pub stake_account: Account<'info, StakeAccount>,
    
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub governance_vault: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
}

#[account]
pub struct Governance {
    pub authority: Pubkey,
    pub arkly_mint: Pubkey,
    pub min_proposal_stake: u64,
    pub voting_period: i64,
    pub execution_delay: i64,
    pub proposal_count: u64,
    pub total_staked: u64,
}

impl Governance {
    pub const LEN: usize = 32 + 32 + 8 + 8 + 8 + 8 + 8;
}

#[account]
pub struct Proposal {
    pub id: u64,
    pub proposer: Pubkey,
    pub title: String,
    pub description: String,
    pub proposal_type: ProposalType,
    pub execution_data: Vec<u8>,
    pub votes_for: u64,
    pub votes_against: u64,
    pub status: ProposalStatus,
    pub created_at: i64,
    pub voting_ends_at: i64,
    pub execution_eta: i64,
}

impl Proposal {
    pub const LEN: usize = 8 + 32 + 4 + 100 + 4 + 500 + 1 + 4 + 256 + 8 + 8 + 1 + 8 + 8 + 8;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum ProposalType {
    ParameterChange,
    TreasurySpend,
    ProtocolUpgrade,
    PropertyListing,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum ProposalStatus {
    Active,
    Queued,
    Executed,
    Defeated,
    Expired,
}

#[account]
pub struct VoterRecord {
    pub proposal: Pubkey,
    pub voter: Pubkey,
    pub has_voted: bool,
    pub support: bool,
    pub voting_power: u64,
    pub voted_at: i64,
}

impl VoterRecord {
    pub const LEN: usize = 32 + 32 + 1 + 1 + 8 + 8;
}

#[account]
pub struct StakeAccount {
    pub user: Pubkey,
    pub staked_amount: u64,
    pub last_stake_time: i64,
}

impl StakeAccount {
    pub const LEN: usize = 32 + 8 + 8;
}

#[event]
pub struct ProposalCreated {
    pub proposal_id: u64,
    pub proposer: Pubkey,
    pub title: String,
    pub proposal_type: ProposalType,
    pub voting_ends_at: i64,
}

#[event]
pub struct VoteCast {
    pub proposal_id: u64,
    pub voter: Pubkey,
    pub support: bool,
    pub voting_power: u64,
    pub timestamp: i64,
}

#[event]
pub struct ProposalQueued {
    pub proposal_id: u64,
    pub execution_eta: i64,
}

#[event]
pub struct ProposalExecuted {
    pub proposal_id: u64,
    pub timestamp: i64,
}

#[event]
pub struct ProposalDefeated {
    pub proposal_id: u64,
}

#[event]
pub struct TokensStaked {
    pub user: Pubkey,
    pub amount: u64,
    pub total_staked: u64,
    pub timestamp: i64,
}

#[event]
pub struct TokensUnstaked {
    pub user: Pubkey,
    pub amount: u64,
    pub remaining_staked: u64,
    pub timestamp: i64,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Insufficient stake to create proposal")]
    InsufficientStake,
    #[msg("Voting period has ended")]
    VotingPeriodEnded,
    #[msg("Proposal is not active")]
    ProposalNotActive,
    #[msg("User has already voted on this proposal")]
    AlreadyVoted,
    #[msg("Voting period has not ended")]
    VotingPeriodNotEnded,
    #[msg("Proposal is not queued for execution")]
    ProposalNotQueued,
    #[msg("Execution delay has not passed")]
    ExecutionDelayNotPassed,
    #[msg("Insufficient staked amount")]
    InsufficientStakedAmount,
}
