# Bits - Encrypted Content Marketplace

A decentralized content marketplace where creators can upload encrypted content and users can pay for access. Think "Web3 OnlyFans" - democratized and encrypted.

## Features

- 🔐 Client-side encryption before upload
- 💳 Stripe payments (crypto coming soon)
- 📦 S3-compatible storage
- 🔑 Magic link authentication
- ⚡ Fast TypeScript monorepo

## Quick Start

### Prerequisites

- [devenv](https://devenv.sh/) installed
- [aws-vault](https://github.com/99designs/aws-vault) configured
- [1Password CLI](https://1password.com/downloads/command-line/) installed
- AWS account (using invetica-bits)
- Stripe account for payments

### AWS Setup

Configure aws-vault for the invetica-bits account:
```bash
aws-vault add invetica-bits
# Enter your AWS access key and secret key for the invetica-bits account
```

### Infrastructure Setup

1. Deploy the AWS infrastructure:
```bash
devenv shell
tf-init    # Initialize Terraform
tf-plan    # Review changes
tf-apply   # Create resources
```

2. Store AWS credentials in 1Password:
```bash
tf-store-secrets  # Automatically saves Terraform outputs to 1Password
```

### Application Setup

1. Clone and enter the dev environment:
```bash
git clone <repo>
cd bits
devenv shell
```

2. Set up 1Password entries (see `docs/1password-setup.md`):
```bash
# Create required 1Password entries for Stripe, Email, and Secrets
op item create --category="API Credential" --title="Bits Dev Stripe" ...
```

3. Start all services:
```bash
devenv up
```

This will start:
- PostgreSQL database
- API server on http://localhost:4444
- Web frontend on http://localhost:5173

## Project Structure

```
bits/
├── packages/
│   ├── shared/     # Shared types and crypto functions
│   ├── api/        # Express backend API
│   └── web/        # React frontend
├── devenv.nix      # Development environment
└── .env.example    # Environment variables template
```

## Development

### Running individual services

```bash
# API only
cd packages/api && pnpm dev

# Frontend only
cd packages/web && pnpm dev

# Build all packages
pnpm build

# Type check all packages
pnpm typecheck
```

### Database

PostgreSQL is automatically configured by devenv with:
- Database: `bits_dev`
- User: `bits`
- Password: `please`

## MVP Checklist

- [x] Monorepo setup with pnpm workspaces
- [x] Shared encryption library
- [x] Express API with TypeScript
- [x] PostgreSQL database schema
- [x] Magic link authentication
- [x] Content upload endpoints
- [x] Stripe payment integration
- [x] React frontend with Vite
- [x] Client-side encryption
- [x] Browse and purchase flow
- [ ] S3/R2 storage configuration
- [ ] Email provider setup
- [ ] Production deployment

## Next Steps

1. Configure S3 bucket and add credentials to `.env`
2. Set up SMTP for magic links (or use a service like SendGrid)
3. Add Stripe API keys
4. Deploy to production (Vercel/Railway recommended)

## Infrastructure

The application uses Terraform to manage AWS resources in the `invetica-bits` account:

- **S3 Bucket**: For encrypted content storage
- **IAM User**: Application-specific credentials with minimal permissions
- **Backend State**: Stored in S3 with DynamoDB locking

All infrastructure is defined in the `iac/` directory.

## Security Notes

- All content is encrypted client-side before upload
- Encryption keys are only delivered after payment
- Platform takes 10% fee on all transactions
- Magic links expire after 15 minutes
- Secrets are managed via 1Password CLI
- AWS credentials are isolated to specific IAM users