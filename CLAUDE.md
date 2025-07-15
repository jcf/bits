# Bits - Project Context for Claude

## Project Overview

Bits is an encrypted content marketplace where creators can upload encrypted content and users pay for access. Think "Web3 OnlyFans" but democratized and encrypted. The project is built as a TypeScript monorepo using pnpm workspaces.

## Current Architecture

### Tech Stack
- **Frontend**: React with Vite, Tailwind CSS, Zustand for state
- **Backend**: Express.js with TypeScript
- **Database**: PostgreSQL (managed by devenv)
- **Storage**: AWS S3 (invetica-bits account)
- **Payments**: Stripe (fiat) - crypto coming soon
- **Encryption**: Web Crypto API (AES-GCM)
- **Auth**: Magic links and wallet signatures
- **Secrets**: 1Password CLI integration
- **Infrastructure**: Terraform for AWS resources

### Project Structure
```
bits/
├── packages/
│   ├── shared/         # Shared types and crypto functions
│   ├── api/           # Express backend
│   └── web/           # React frontend
├── iac/               # Terraform infrastructure
├── devenv.nix         # Development environment
└── .env.1password     # Secret mappings
```

### Key Features Implemented
- ✅ Client-side encryption before upload
- ✅ Encrypted content storage in S3
- ✅ Magic link authentication
- ✅ Stripe payment integration
- ✅ Content browsing and purchase flow
- ✅ Webhook handling for payment confirmation
- ✅ 1Password integration for all secrets

## Infrastructure Details

### AWS Resources (invetica-bits account)
- S3 bucket: `bits-dev-content` (encrypted, versioned)
- IAM user: `bits-dev-app` (minimal permissions)
- DynamoDB table: `terraform-state-lock` (for Terraform state)
- Backend state bucket: `invetica-bits-terraform-state`

### 1Password Vault Structure
All secrets are stored in the "Bits" vault:
- `Bits Dev AWS` - AWS credentials from Terraform
- `Bits Dev Stripe` - Stripe API keys and webhook secret
- `Bits Dev Email` - SMTP configuration
- `Bits Dev Secrets` - JWT and magic link secrets

## Development Workflow

### Running the Application
```bash
devenv shell        # Enter development environment
devenv up          # Start all services (PostgreSQL, API, Web)
```

### Terraform Commands
```bash
tf-init           # Initialize Terraform
tf-plan           # Review infrastructure changes
tf-apply          # Deploy infrastructure
tf-store-secrets  # Save AWS credentials to 1Password
```

### Secret Management
```bash
setup-secrets         # Set up all secrets interactively
create-stripe-secrets # Add Stripe credentials
create-app-secrets    # Generate JWT secrets
create-email-config   # Configure SMTP
```

## Future Roadmap: Decentralization & E2EE

### Phase 1: Stablecoin Integration (Next Priority)
- [ ] Add USDC payment option alongside Stripe
- [ ] Integrate wagmi/viem for wallet connections
- [ ] Smart contract for payment escrow
- [ ] Multi-chain support (Ethereum, Polygon, Arbitrum)

### Phase 2: Enhanced E2EE
- [ ] Implement key exchange protocol
- [ ] Add group encryption for shared access
- [ ] Client-side key derivation from wallet signatures
- [ ] Zero-knowledge proofs for purchase verification

### Phase 3: Decentralized Storage
- [ ] IPFS integration for content storage
- [ ] Filecoin for long-term archival
- [ ] Content addressing with CIDs
- [ ] P2P content delivery

### Phase 4: Full Decentralization
- [ ] DAO governance for platform decisions
- [ ] Decentralized identity (DID) integration
- [ ] Content moderation through community consensus
- [ ] Revenue sharing smart contracts

## Technical Decisions & Rationale

### Why Web Crypto API?
- Browser-native, no dependencies
- Secure key generation and storage
- Good performance for large files
- Works with both web and React Native

### Why PostgreSQL for MVP?
- Fast development with familiar tooling
- Easy migration path to decentralized storage
- Good for metadata while content is encrypted

### Why Stripe First?
- Fastest path to revenue
- Users familiar with card payments
- Crypto can be added as alternative

### Why 1Password?
- Secure secret management
- Team sharing capabilities
- CLI integration for automation
- Audit trail for compliance

## Security Considerations

1. **Encryption Keys**: Never stored on server, only transmitted after payment
2. **Content Access**: Server never sees decrypted content
3. **Payment Security**: Stripe handles PCI compliance
4. **AWS Access**: IAM user has minimal S3 permissions only
5. **Database**: No sensitive content stored, only metadata

## Known Issues & TODOs

### Immediate Fixes Needed
- [ ] Add proper error handling for S3 uploads
- [ ] Implement retry logic for failed payments
- [ ] Add content type validation
- [ ] Set up email provider (currently SMTP not configured)

### Performance Optimizations
- [ ] Implement chunked upload for large files
- [ ] Add CDN for content delivery
- [ ] Cache decryption keys in IndexedDB
- [ ] Optimize bundle size with code splitting

### User Experience
- [ ] Add upload progress indicators
- [ ] Implement content previews
- [ ] Add creator profiles
- [ ] Build analytics dashboard

## Local Development Tips

1. **Stripe Webhooks**: Use `stripe listen --forward-to localhost:4444/webhook/stripe`
2. **S3 Alternative**: Can use MinIO locally if needed
3. **Email Testing**: Use MailHog or similar for local SMTP
4. **Database Access**: `psql postgresql://bits:please@localhost:5432/bits_dev`

## Deployment Considerations

- Frontend: Vercel or Cloudflare Pages
- Backend: Railway or Fly.io
- Database: Supabase or Neon
- Consider edge functions for decryption key delivery

## Business Model Evolution

Current: 10% platform fee on all transactions

Future considerations:
- Creator staking for reduced fees
- Governance token for platform decisions
- NFT receipts for purchases
- Subscription tiers for creators

## Integration Points

Ready for:
- Wallet connections (MetaMask, WalletConnect)
- IPFS pinning services (Pinata, Web3.Storage)
- ENS/Lens Protocol for creator profiles
- Push Protocol for notifications

## Remember

- All secrets go through 1Password
- Never commit sensitive data
- Use atomic commits with clear messages
- Test encryption/decryption thoroughly
- Keep decentralization as north star