# 1Password Setup for Bits

This document outlines the 1Password entries needed for the Bits application.

## Required 1Password Entries

### 1. AWS Credentials (Auto-created by Terraform)

- **Title**: `Bits Dev AWS`
- **Category**: API Credential
- **Vault**: Bits
- **Fields**:
  - `AWS_ACCESS_KEY_ID`: (Created by terraform)
  - `AWS_SECRET_ACCESS_KEY`: (Created by terraform)
  - `S3_BUCKET`: (Created by terraform)

### 2. Stripe API Keys

- **Title**: `Bits Dev Stripe`
- **Category**: API Credential
- **Vault**: Bits
- **Fields**:
  - `secret_key`: Your Stripe secret key (sk*test*...)
  - `webhook_secret`: Your Stripe webhook secret (whsec\_...)

### 3. Email/SMTP Configuration

- **Title**: `Bits Dev Email`
- **Category**: Login
- **Vault**: Bits
- **Fields**:
  - `hostname`: smtp.gmail.com (or your SMTP host)
  - `port`: 587
  - `username`: Your SMTP username
  - `password`: Your SMTP password

### 4. Application Secrets

- **Title**: `Bits Dev Secrets`
- **Category**: Secure Note
- **Vault**: Bits
- **Fields**:
  - `jwt_secret`: (Generate a random 32+ character string)
  - `magic_link_secret`: (Generate a random 32+ character string)

## Creating Entries via CLI

```bash
# Create Stripe entry
op item create \
  --category="API Credential" \
  --title="Bits Dev Stripe" \
  --vault="Bits" \
  "secret_key=sk_test_your_key_here" \
  "webhook_secret=whsec_your_secret_here"

# Create Email entry
op item create \
  --category="Login" \
  --title="Bits Dev Email" \
  --vault="Bits" \
  "hostname=smtp.gmail.com" \
  "port=587" \
  "username=your-email@gmail.com" \
  "password=your-app-password"

# Create application secrets
op item create \
  --category="Secure Note" \
  --title="Bits Dev Secrets" \
  --vault="Bits" \
  "jwt_secret=$(openssl rand -base64 32)" \
  "magic_link_secret=$(openssl rand -base64 32)"
```

## Usage

The application uses these secrets via the `.env.1password` file:

```bash
# Run the API with 1Password secrets
op run --env-file=.env.1password -- pnpm dev

# Or use devenv which handles this automatically
devenv up
```

## Production Considerations

For production, create separate 1Password entries:

- `Bits Prod AWS`
- `Bits Prod Stripe`
- `Bits Prod Email`
- `Bits Prod Secrets`

And use a separate `.env.1password.prod` file with production vault references.
