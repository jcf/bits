# Infrastructure as Code

This directory contains Terraform configuration for the Bits application infrastructure.

## Structure

```
iac/
├── modules/           # Reusable Terraform modules
│   └── storage/      # S3 bucket and IAM configuration
├── environments/      # Environment-specific configurations
│   └── dev/          # Development environment
├── backend.tf        # Terraform state backend configuration
├── variables.tf      # Common variables
└── versions.tf       # Provider versions
```

## Prerequisites

1. AWS CLI configured with access to the invetica-bits account
2. aws-vault set up for the invetica-bits profile
3. Terraform >= 1.6

## Backend State

Terraform state is stored in S3 with the following configuration:

- Bucket: `invetica-bits-terraform-state`
- Key: `bits/terraform.tfstate`
- Region: `eu-west-2`
- DynamoDB Table: `terraform-state-lock`

**Note**: The backend bucket must be created manually before running Terraform.

## Usage

### Development Environment

```bash
cd environments/dev

# Initialize Terraform (first time only)
aws-vault exec invetica-bits -- terraform init

# Plan changes
aws-vault exec invetica-bits -- terraform plan

# Apply changes
aws-vault exec invetica-bits -- terraform apply
```

### Using devenv scripts

If you're in the devenv shell:

```bash
tf-init    # Initialize Terraform
tf-plan    # Review changes
tf-apply   # Create resources
```

## Resources Created

### Storage Module

- S3 bucket for content storage (encrypted, versioned)
- IAM user with minimal permissions for S3 access
- CORS configuration for browser uploads

## Adding New Environments

1. Create a new directory under `environments/`
2. Copy `main.tf` and adjust the `environment` local
3. Update bucket names and other environment-specific values
4. Run terraform init and apply

## Security Considerations

- All S3 buckets have encryption enabled by default
- Public access is blocked on all buckets
- IAM users have minimal required permissions
- Access keys should be immediately stored in 1Password after creation
