# Sensitive Files - DO NOT COMMIT TO GITHUB

## Files that should NEVER be pushed to public repositories:

### 🔐 Cryptographic Keys & Wallets
- Any .json files containing private keys
- Solana keypairs (id.json, deployer.json, etc.)
- Wallet files

### 📊 Private Business Data
- Real property addresses and specific details
- Customer data or KYC information
- Financial reports with real numbers
- Internal business documents

### 🔧 Configuration Files
- .env files with real API keys
- Database connection strings
- Third-party service credentials

### 💼 Legal & Compliance
- Real legal contracts
- Compliance audit reports
- Regulatory correspondence

## ✅ What IS safe to push:

### 📝 Open Source Code
- Smart contract source code (.rs files)
- Frontend code (HTML, CSS, JS)
- Documentation and README files
- Test files with mock data

### 🏗️ Project Structure
- Build configurations (Cargo.toml, package.json)
- Development tools setup
- CI/CD configurations

### 📚 Public Documentation
- API documentation
- User guides
- Security best practices

## 🛡️ Security Checklist Before Push:

1. ✅ No private keys or wallet files
2. ✅ No real customer data
3. ✅ No production environment variables
4. ✅ No internal business secrets
5. ✅ Only educational/example data in tests
6. ✅ Legal disclaimers properly included

## 📋 Current Repository Status:

This repository contains:
- ✅ Smart contract source code (educational/development)
- ✅ Frontend demo code
- ✅ Documentation and guides
- ✅ Legal disclaimers and proper licensing
- ✅ Development tools and configuration

All content is suitable for public GitHub repository.
