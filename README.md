# Arkly Capital - Tokenized Real Estate Platform

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Solana](https://img.shields.io/badge/Solana-000000?style=flat&logo=solana&logoColor=00FFA3)](https://solana.com/)
[![Anchor](https://img.shields.io/badge/Anchor-0.28.0-blue)](https://www.anchor-lang.com/)

🏗️ **Production-Ready Smart Contracts for Tokenized Real Estate on Solana**

## ⚠️ Important Legal Notice

**Arkly Capital Ltd.** is an international private company incorporated under the International Business Companies Act, 2016 (Seychelles). 
- **Registered Number**: IBC-247019
- **Registered Office**: Suite 1, Global Capital House, Mont Fleuri, Victoria, Mahé, Seychelles
- **Legal Disclaimer**: This software does not constitute investment advice or an offer to solicit investments.

---

> **Bridging traditional real estate with decentralized finance through tokenized rental properties.**

Arkly Capital democratizes access to premium real estate investments by tokenizing rental properties on the Solana blockchain. Our platform enables fractional ownership, automated yield distribution, and transparent asset management through smart contracts.

## 🏗️ Architecture Overview

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Frontend DApp │    │  Smart Contracts │    │  Real Estate    │
│                 │    │                 │    │    Assets       │
│ • Web Interface │◄──►│ • Property Tokens│◄──►│ • Rental Income │
│ • Wallet Connect│    │ • Yield Distrib. │    │ • Asset Mgmt    │
│ • Portfolio Mgmt│    │ • Governance     │    │ • Legal Wrapper │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## 🚀 Key Features

- **Fractional Ownership**: Own portions of premium real estate starting from $350
- **Automated Yield**: Receive rental income distributions through smart contracts
- **Transparent Management**: On-chain governance and asset performance tracking
- **Cross-Border Access**: Global investment opportunities without traditional barriers
- **Liquid Secondary Market**: Trade property tokens on decentralized exchanges

## 📊 Tokenomics

| Allocation | Tokens | Percentage | Price | Vesting |
|------------|--------|------------|-------|---------|
| **Seed Round** | 12,000,000 | 12% | $0.083 | 6-month cliff, 12-month linear |
| **Public Presale** | 7,500,000 | 7.5% | $0.10 | 100% liquid at TGE |
| **Liquidity Pool** | 10,000,000 | 10% | $0.15 | 50% at TGE, 50% after 30 days |
| **Team & Advisors** | 15,000,000 | 15% | - | 12-month cliff, 24-month linear |
| **Ecosystem/Rewards** | 25,000,000 | 25% | - | Progressive over 36 months |
| **Treasury/Dev** | 20,000,000 | 20% | - | Multi-sig, roadmap-based |
| **Strategic Partners** | 5,000,000 | 5% | - | 6-month cliff, 12-month vesting |
| **Community/Airdrops** | 8,000,000 | 8% | - | Claim-based distribution |

**Total Supply**: 100,000,000 $ARKLY  
**Hardcap**: $750,000

## 🛠️ Technology Stack

### Smart Contracts
- **Framework**: Anchor (Solana)
- **Language**: Rust
- **Programs**: Property tokenization, yield distribution, governance

### Frontend
- **Framework**: Next.js + TypeScript
- **Wallet**: Solana Wallet Adapter
- **UI**: Tailwind CSS + Custom Components
- **Web3**: @solana/web3.js

### Backend Infrastructure
- **API**: Node.js + Express
- **Database**: PostgreSQL + Redis
- **Monitoring**: DataDog, Sentry
- **Infrastructure**: AWS + CDN

## 📁 Repository Structure

```
arkly-capital/
├── programs/                 # Solana smart contracts
│   ├── arkly-token/         # $ARKLY token program
│   ├── property-vault/      # Property tokenization
│   ├── yield-distributor/   # Automated yield distribution
│   └── governance/          # DAO governance contracts
├── app/                     # Frontend application
│   ├── components/          # React components
│   ├── pages/              # Next.js pages
│   ├── hooks/              # Custom React hooks
│   └── utils/              # Utility functions
├── api/                     # Backend services
│   ├── routes/             # API endpoints
│   ├── services/           # Business logic
│   └── models/             # Data models
├── tests/                   # Test suites
├── docs/                   # Documentation
└── scripts/                # Deployment scripts
```

## 🚀 Quick Start

### Prerequisites
- Node.js 18+
- Rust 1.70+
- Solana CLI 1.16+
- Anchor CLI 0.28+

### Installation

1. **Clone the repository**
   ```bash
   git clone https://github.com/arklycap/arkly-capital.git
   cd arkly-capital
   ```

2. **Install dependencies**
   ```bash
   npm install
   cd programs && cargo build-bpf
   ```

3. **Configure environment**
   ```bash
   cp .env.example .env
   # Edit .env with your configuration
   ```

4. **Deploy smart contracts** (Devnet)
   ```bash
   anchor build
   anchor deploy
   ```

5. **Start development server**
   ```bash
   npm run dev
   ```

Visit `http://localhost:3000` to see the application.

## 🔐 Security

### Smart Contract Audits
- [ ] Trail of Bits - Q1 2025
- [ ] Certik - Q1 2025
- [ ] Halborn - Q2 2025

### Security Features
- Multi-signature treasury management
- Time-locked contract upgrades
- Emergency pause functionality
- Automated yield verification
- KYC/AML compliance integration

### Bug Bounty Program
We offer rewards up to **$50,000** for critical vulnerabilities. See [SECURITY.md](./SECURITY.md) for details.

## 🏛️ Governance

Arkly operates as a DAO with the following governance structure:

- **Proposal Threshold**: 100,000 $ARKLY
- **Voting Period**: 7 days
- **Quorum**: 10% of circulating supply
- **Timelock**: 24 hours for execution

### Governance Powers
- Property acquisition proposals
- Fee structure modifications
- Treasury fund allocation
- Protocol upgrades

## 📈 Roadmap

### Q1 2025
- [ ] Smart contract development
- [ ] Security audits
- [ ] Public presale launch
- [ ] MVP platform release

### Q2 2025
- [ ] First property tokenization
- [ ] Yield distribution implementation
- [ ] Mobile app launch
- [ ] Partnership integrations

### Q3 2025
- [ ] Multi-asset portfolio
- [ ] Advanced analytics dashboard
- [ ] Cross-chain bridge (Ethereum)
- [ ] Institutional partnerships

### Q4 2025
- [ ] Global expansion
- [ ] REITs integration
- [ ] Advanced DeFi features
- [ ] Mobile trading suite

## 🤝 Contributing

We welcome contributions from the community! Please read our [Contributing Guidelines](./CONTRIBUTING.md) before submitting pull requests.

### Development Workflow
1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## 📄 Legal & Compliance

Arkly Capital operates under the regulatory framework of international jurisdictions:

- **Entity**: Arkly Capital Ltd. (Seychelles IBC)
- **Registration**: IBC-247019
- **Compliance**: KYC/AML procedures implemented
- **Disclaimer**: Not available to US persons or restricted jurisdictions

## 📞 Contact & Support

- **Website**: [arkly.capital](https://arkly.capital)
- **Email**: team@arkly.capital
- **Twitter**: [@arklycap](https://x.com/arklycap)
- **Telegram**: [t.me/arklycap](https://t.me/arklycap)
- **GitHub**: [github.com/arklycap](https://github.com/arklycap)

## 📋 License

This project is licensed under the MIT License - see the [LICENSE](./LICENSE) file for details.

---

**⚠️ Risk Disclaimer**: Investing in tokenized real estate involves significant risks. Past performance is not indicative of future results. Please read our full risk disclosure before participating.

**🔒 Security Notice**: Never share your private keys or seed phrases. Arkly team will never ask for your private keys.
