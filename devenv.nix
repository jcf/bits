{pkgs, ...}: {
  packages = with pkgs; [
    _1password-cli
    stripe-cli
    terraform
  ];

  # Non-sensitive environment variables
  env.AWS_REGION = "eu-west-2";
  env.DATABASE_URL = "postgresql://bits:please@localhost:5432/bits_dev";
  env.PORT = "4444";
  env.S3_BUCKET = "bits-dev-content";
  env.WEB_URL = "http://localhost:5173";

  # Development-only secrets (will be overridden by 1Password in production)
  env.JWT_SECRET = "dev-jwt-secret-change-in-production";
  env.MAGIC_LINK_SECRET = "dev-magic-link-secret-change-in-production";

  languages.javascript.enable = true;
  languages.javascript.pnpm.enable = true;
  languages.javascript.pnpm.install.enable = true;

  process.manager.implementation = "process-compose";
  process.managers.process-compose.tui.enable = false;

  services.postgres = {
    enable = true;

    package = pkgs.postgresql_16;

    listen_addresses = "127.0.0.1";
    initialDatabases = [
      {
        name = "bits_dev";
        user = "bits";
        pass = "please";
      }
      {
        name = "bits_test";
        user = "bits";
        pass = "please";
      }
    ];
  };

  processes = {
    api = {
      exec = ''
        cd packages/api && \
        op run --env-file=../../.env.1password -- pnpm dev
      '';
      process-compose = {
        depends_on.postgres.condition = "process_healthy";
      };
    };
    web = {
      exec = ''
        cd packages/web && \
        op run --env-file=../../.env.1password -- \
        sh -c 'VITE_STRIPE_PUBLISHABLE_KEY=$STRIPE_PUBLISHABLE_KEY pnpm dev'
      '';
    };
  };

  # Scripts for common tasks
  scripts = {
    # Deploy infrastructure
    tf-init.exec = ''
      cd iac/environments/dev && \
      aws-vault exec bits -- terraform init
    '';

    tf-plan.exec = ''
      cd iac/environments/dev && \
      aws-vault exec bits -- terraform plan
    '';

    tf-apply.exec = ''
      cd iac/environments/dev && \
      aws-vault exec bits -- terraform apply
    '';

    # Store terraform outputs in 1Password
    tf-store-secrets.exec = ''
      echo "Storing Terraform outputs in 1Password..."
      cd iac/environments/dev

      # Get outputs
      BUCKET=$(aws-vault exec bits -- terraform output -raw s3_bucket_name)
      ACCESS_KEY=$(aws-vault exec bits -- terraform output -raw aws_access_key_id)
      SECRET_KEY=$(aws-vault exec bits -- terraform output -raw aws_secret_access_key)

      # Create 1Password item (you'll need to adjust vault name)
      op item create \
        --category="API Credential" \
        --title="Bits Dev AWS" \
        --vault="Bits" \
        "AWS_ACCESS_KEY_ID=$ACCESS_KEY" \
        "AWS_SECRET_ACCESS_KEY=$SECRET_KEY" \
        "S3_BUCKET=$BUCKET"
    '';

    # Create Stripe secrets in 1Password
    create-stripe-secrets.exec = ''
      echo "Please enter your Stripe test mode secret key (sk_test_...):"
      read -s STRIPE_SECRET
      echo

      echo "Please enter your Stripe test mode publishable key (pk_test_...):"
      read STRIPE_PUBLISHABLE
      echo

      echo "Please enter your Stripe webhook signing secret (whsec_...):"
      echo "(You can get this from Stripe Dashboard > Developers > Webhooks)"
      read -s STRIPE_WEBHOOK
      echo

      # Create 1Password item
      op item create \
        --category="API Credential" \
        --title="Bits Dev Stripe" \
        --vault="Bits" \
        "secret_key=$STRIPE_SECRET" \
        "publishable_key=$STRIPE_PUBLISHABLE" \
        "webhook_secret=$STRIPE_WEBHOOK"

      echo "Stripe credentials stored in 1Password!"
    '';

    # Create application secrets in 1Password
    create-app-secrets.exec = ''
      echo "Generating application secrets..."

      JWT_SECRET=$(openssl rand -base64 32)
      MAGIC_LINK_SECRET=$(openssl rand -base64 32)

      op item create \
        --category="Secure Note" \
        --title="Bits Dev Secrets" \
        --vault="Bits" \
        "jwt_secret=$JWT_SECRET" \
        "magic_link_secret=$MAGIC_LINK_SECRET"

      echo "Application secrets stored in 1Password!"
    '';

    # Create email configuration in 1Password
    create-email-config.exec = ''
      echo "Enter SMTP configuration for magic links:"
      echo "SMTP Host (e.g., smtp.gmail.com):"
      read SMTP_HOST

      echo "SMTP Port (e.g., 587):"
      read SMTP_PORT

      echo "SMTP Username (your email):"
      read SMTP_USER

      echo "SMTP Password (app-specific password for Gmail):"
      read -s SMTP_PASS
      echo

      op item create \
        --category="Login" \
        --title="Bits Dev Email" \
        --vault="Bits" \
        "hostname=$SMTP_HOST" \
        "port=$SMTP_PORT" \
        "username=$SMTP_USER" \
        "password=$SMTP_PASS"

      echo "Email configuration stored in 1Password!"
    '';

    # Setup all secrets at once
    setup-secrets.exec = ''
      echo "=== Setting up all secrets for Bits ==="
      echo

      # Check if AWS secrets exist
      if ! op item get "Bits Dev AWS" --vault="Bits" >/dev/null 2>&1; then
        echo "AWS secrets not found. Running tf-store-secrets..."
        tf-store-secrets
      else
        echo "✓ AWS secrets already configured"
      fi

      # Check if Stripe secrets exist
      if ! op item get "Bits Dev Stripe" --vault="Bits" >/dev/null 2>&1; then
        echo
        echo "Setting up Stripe..."
        create-stripe-secrets
      else
        echo "✓ Stripe secrets already configured"
      fi

      # Check if app secrets exist
      if ! op item get "Bits Dev Secrets" --vault="Bits" >/dev/null 2>&1; then
        echo
        echo "Setting up application secrets..."
        create-app-secrets
      else
        echo "✓ Application secrets already configured"
      fi

      # Check if email config exists
      if ! op item get "Bits Dev Email" --vault="Bits" >/dev/null 2>&1; then
        echo
        echo "Setting up email configuration..."
        create-email-config
      else
        echo "✓ Email configuration already configured"
      fi

      echo
      echo "=== All secrets configured! ==="
      echo
      echo "To configure Stripe webhooks:"
      echo "1. Go to https://dashboard.stripe.com/test/webhooks"
      echo "2. Add endpoint URL: https://your-domain.com/webhook/stripe"
      echo "3. Select events: payment_intent.succeeded"
      echo "4. Copy the signing secret and update it in 1Password"
      echo
      echo "For local development with Stripe CLI:"
      echo "stripe listen --forward-to localhost:4444/webhook/stripe"
    '';
  };
}
