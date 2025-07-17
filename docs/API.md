# Arkly Capital Smart Contract API Documentation

## Overview

The Arkly Capital platform consists of four main smart contracts deployed on the Solana blockchain:

1. **ARKLY Token** - The native utility token with tokenomics and vesting
2. **Property Vault** - Tokenization of real estate properties
3. **Governance** - Decentralized governance for protocol decisions
4. **Yield Distributor** - Automated rental yield distribution

## ARKLY Token Contract

### Program ID
`ARK1yT0k3nM1nt1ngPr3s4l3V3st1ngR3w4rdD1str1but10n`

### Key Functions

#### `initialize_token(tokenomics_allocations: TokenomicsAllocations)`
Initializes the ARKLY token with predefined tokenomics.

**Parameters:**
- `tokenomics_allocations`: Distribution of 100M total supply across 8 categories

**Accounts:**
- `token_config`: PDA for token configuration
- `mint`: ARKLY token mint
- `treasury`: Treasury PDA
- `authority`: Token authority

#### `purchase_presale(usdc_amount: u64)`
Allows users to purchase ARKLY tokens during presale.

**Parameters:**
- `usdc_amount`: Amount of USDC to spend

**Accounts:**
- `token_config`: Token configuration
- `presale_account`: User's presale record
- `buyer_token_account`: User's ARKLY token account
- `buyer_usdc_account`: User's USDC account
- `treasury_usdc`: Treasury USDC account

#### `claim_vested_tokens()`
Claims vested tokens for team, advisors, and other allocated parties.

**Accounts:**
- `token_config`: Token configuration
- `vesting_account`: User's vesting record
- `recipient_token_account`: Recipient's token account

### Events

- `TokenInitialized`: Emitted when token is initialized
- `PresalePurchase`: Emitted on presale purchases
- `VestedTokensClaimed`: Emitted when vested tokens are claimed

## Property Vault Contract

### Program ID
`PR0P3rtyV4u1tM1ntT0k3n1z4t10nPr0gr4mId3nt1f13r`

### Key Functions

#### `initialize_property(property_id: String, total_tokens: u64, token_price: u64, expected_annual_yield: u16, property_details: PropertyDetails)`
Creates a new property vault for tokenization.

**Parameters:**
- `property_id`: Unique identifier for the property
- `total_tokens`: Total tokens representing the property
- `token_price`: Price per token in USDC
- `expected_annual_yield`: Expected yield in basis points
- `property_details`: Metadata about the property

#### `purchase_property_tokens(amount: u64)`
Purchase tokens representing ownership in a property.

**Parameters:**
- `amount`: Number of property tokens to purchase

#### `claim_yield()`
Claim rental yield based on token ownership.

### Events

- `PropertyInitializedEvent`: Property vault created
- `TokensPurchasedEvent`: Property tokens purchased
- `YieldClaimedEvent`: Yield claimed by token holder

## Governance Contract

### Program ID
`G0v3rn4nc3V0t1ngD4oM4n4g3m3ntSm4rtC0ntr4ct1d`

### Key Functions

#### `initialize_governance(min_proposal_stake: u64, voting_period: i64, execution_delay: i64)`
Initialize the governance system.

**Parameters:**
- `min_proposal_stake`: Minimum ARKLY tokens needed to create proposals
- `voting_period`: Duration of voting in seconds
- `execution_delay`: Delay before execution in seconds

#### `create_proposal(title: String, description: String, proposal_type: ProposalType, execution_data: Vec<u8>)`
Create a new governance proposal.

**Parameters:**
- `title`: Proposal title
- `description`: Detailed description
- `proposal_type`: Type of proposal (ParameterChange, TreasurySpend, etc.)
- `execution_data`: Encoded execution instructions

#### `vote(support: bool)`
Vote on an active proposal.

**Parameters:**
- `support`: True for yes, false for no

#### `execute_proposal()`
Execute a proposal that has passed and waited the execution delay.

### Proposal Types

- `ParameterChange`: Modify protocol parameters
- `TreasurySpend`: Spend from treasury
- `ProtocolUpgrade`: Upgrade protocol contracts
- `PropertyListing`: Add new properties to platform

### Events

- `ProposalCreated`: New proposal created
- `VoteCast`: Vote submitted
- `ProposalQueued`: Proposal queued for execution
- `ProposalExecuted`: Proposal executed

## Yield Distributor Contract

### Program ID
`Y13ldD1str1but0rR3nt4lR3v3nu3Sh4r1ngSm4rtC0ntr4ct`

### Key Functions

#### `initialize_yield_pool(pool_id: String, property_mint: Pubkey, distribution_frequency: DistributionFrequency)`
Create a yield distribution pool for a property.

**Parameters:**
- `pool_id`: Unique identifier for the pool
- `property_mint`: Mint of the property tokens
- `distribution_frequency`: How often yields are distributed

#### `deposit_yield(amount: u64, yield_period: String)`
Deposit rental income for distribution.

**Parameters:**
- `amount`: USDC amount to deposit
- `yield_period`: Period this yield covers

#### `create_distribution_snapshot(snapshot_id: String, total_tokens_eligible: u64, yield_amount: u64)`
Create a snapshot for yield distribution.

#### `claim_yield(token_balance: u64, merkle_proof: Vec<[u8; 32]>)`
Claim yield based on token balance at snapshot.

**Parameters:**
- `token_balance`: Token balance at snapshot time
- `merkle_proof`: Merkle proof for verification

### Distribution Frequencies

- `Monthly`: Every month
- `Quarterly`: Every 3 months
- `SemiAnnually`: Every 6 months
- `Annually`: Every year

### Events

- `YieldPoolCreated`: Pool initialized
- `YieldDeposited`: Yield deposited
- `DistributionCreated`: Snapshot created
- `YieldClaimed`: Yield claimed by user

## Error Codes

### Common Errors

- `Unauthorized`: Caller not authorized for operation
- `InsufficientFunds`: Not enough tokens/USDC
- `InvalidAmount`: Amount is zero or negative
- `AccountNotFound`: Required account not found

### Token-Specific Errors

- `PresaleNotActive`: Presale period has ended
- `VestingNotStarted`: Vesting period hasn't begun
- `NoTokensToVest`: No vested tokens available

### Property-Specific Errors

- `PropertyNotActive`: Property not available for purchase
- `InsufficientTokenSupply`: Not enough tokens available
- `InvalidPropertyData`: Property metadata invalid

### Governance-Specific Errors

- `InsufficientStake`: Not enough staked tokens
- `VotingPeriodEnded`: Voting period has closed
- `ProposalNotActive`: Proposal not in voting state
- `AlreadyVoted`: User already voted on proposal

### Yield-Specific Errors

- `PoolNotActive`: Yield pool is paused/closed
- `DistributionExpired`: Distribution period ended
- `AlreadyClaimed`: User already claimed yield
- `InvalidProof`: Merkle proof verification failed

## Integration Examples

### TypeScript SDK Usage

```typescript
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { ArklyToken } from "./types/arkly_token";

// Initialize program
const program = anchor.workspace.ArklyToken as Program<ArklyToken>;

// Purchase presale tokens
await program.methods
  .purchasePresale(new anchor.BN(1000 * 10**6)) // 1000 USDC
  .accounts({
    // ... account setup
  })
  .rpc();
```

### Python Integration

```python
from solana.rpc.api import Client
from anchorpy import Program, Provider

# Connect to Solana
client = Client("https://api.devnet.solana.com")
provider = Provider(client, wallet)

# Load program
program = Program.load("./target/idl/arkly_token.json", provider)

# Call functions
await program.rpc.purchase_presale(
    1000_000_000,  # 1000 USDC
    ctx=Context(accounts={...})
)
```

## Security Considerations

1. **Access Control**: All admin functions require proper authority verification
2. **Reentrancy Protection**: Critical functions use checks-effects-interactions pattern
3. **Integer Overflow**: All arithmetic operations use safe math
4. **Account Validation**: All accounts are validated before use
5. **Time-based Logic**: Uses on-chain time for all time-dependent operations

## Testing

The project includes comprehensive tests for all contracts:

```bash
# Run all tests
anchor test

# Run specific test file
anchor test --file tests/arkly-capital.ts

# Test on devnet
anchor test --provider.cluster devnet
```

## Deployment

Use the provided deployment script:

```bash
# Deploy to devnet
./scripts/deploy.sh devnet

# Deploy to mainnet (use with caution)
./scripts/deploy.sh mainnet
```

## Support

For technical support or questions:
- GitHub Issues: [github.com/arkly-capital/arkly-capital](https://github.com/arkly-capital/arkly-capital)
- Documentation: [docs.arkly.capital](https://docs.arkly.capital)
- Discord: [discord.gg/arkly](https://discord.gg/arkly)
