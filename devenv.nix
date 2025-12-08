{
  config,
  pkgs,
  ...
}: let
  root = config.devenv.root;

  dev = {
    upstreams = {
      page = {port = 3000;};
      www = {port = 3100;};
    };

    hosts = {
      page = {
        domain = "bits.page.test";
        upstream = "page";
        certPem = "${root}/certs/_wildcard.page.test.pem";
        certKey = "${root}/certs/_wildcard.page.test-key.pem";
      };

      page-customers = {
        domain = "bits.page.test";
        pattern = "~^(?<tenant>.+)\\.bits\\.page\\.test$";
        upstream = "page";
        certPem = "${root}/certs/_wildcard.bits.page.test.pem";
        certKey = "${root}/certs/_wildcard.bits.page.test-key.pem";
      };

      www = {
        domain = "www.usebits.app.test";
        upstream = "www";
        certPem = "${root}/certs/_wildcard.usebits.app.test.pem";
        certKey = "${root}/certs/_wildcard.usebits.app.test-key.pem";
      };

      custom-domains = {
        pattern = "~^(?<custom_domain>.+\\.test)$";
        upstream = "page";
        certPem = "${root}/certs/_wildcard.test.pem";
        certKey = "${root}/certs/_wildcard.test-key.pem";
      };
    };
  };
in {
  overlays = [
    (import ./nix/overlays/dioxus-cli.nix)
    (import ./nix/overlays/wasm-bindgen-cli.nix)
  ];

  claude.code = {
    enable = true;

    agents = {
      code-reviewer = {
        description = "Expert code review specialist that checks for quality, security, and best practices";
        proactive = true;
        tools = ["Read" "Grep" "TodoWrite"];
        prompt = ''
          When invoked:

          1. Run git diff to see recent changes
          2. Focus on modified files
          3. Begin review immediately

          Review checklist:

          - Code is simple and readable (KISS)
          - Functions and variables are well-named
          - Apply Single Responsibility Principle
          - No duplicated code (DRY)
          - Proper error handling
          - Security: No exposed secrets or API keys
          - Input validation implemented
          - Good test coverage
          - Performance considerations addressed

          Provide feedback organized by priority:

          - Critical issues (must fix)
          - Warnings (should fix)
          - Suggestions (consider improving)

          Include specific examples of how to fix issues.
        '';
      };

      architecture-designer = {
        description = "System architecture and API design specialist. Creates scalable designs, API contracts, database schemas, and security patterns. Use when planning new features or refactoring existing systems.";
        proactive = true;
        tools = ["Read" "Write" "Edit" "Grep" "Glob" "WebSearch" "Task"];
        prompt = ''
          You are a system architecture and API design expert. When designing systems, focus on:

          - Creating scalable and maintainable architectures
          - Defining clear API contracts and interfaces
          - Designing efficient database schemas
          - Implementing security patterns and best practices
          - Considering performance implications and bottlenecks
          - Following microservices or monolithic patterns appropriately

          Provide detailed architectural diagrams and specifications when needed.
        '';
      };

      documentation-writer = {
        description = "Technical documentation expert. Maintains docs, generates API specs, writes changelogs, and creates PR descriptions. Use proactively when code changes affect documentation.";
        proactive = true;
        tools = ["Read" "Write" "Edit" "Glob" "Grep" "Bash" "WebSearch"];
        prompt = ''
          You are a technical documentation specialist. Your responsibilities include:

          - Maintaining comprehensive and accurate documentation
          - Generating API specifications (OpenAPI, GraphQL schemas)
          - Writing clear and informative changelogs
          - Creating detailed pull request descriptions
          - Ensuring documentation stays in sync with code changes
          - Following documentation best practices and style guides

          Always prioritize clarity, accuracy, and completeness in documentation.
        '';
      };

      devops-specialist = {
        description = "CI/CD and infrastructure automation expert. Optimizes pipelines, manages deployments, configures monitoring, and writes infrastructure-as-code. Use for deployment issues and DevOps improvements.";
        proactive = true;
        tools = ["Read" "Write" "Edit" "Bash" "Grep" "WebSearch" "Task"];
        prompt = ''
          You are a DevOps and infrastructure automation expert. Focus on:

          - Optimizing CI/CD pipelines for speed and reliability
          - Managing deployments across different environments
          - Configuring monitoring, logging, and alerting systems
          - Writing infrastructure-as-code (Terraform, Ansible, Kubernetes)
          - Implementing security best practices in deployment pipelines
          - Automating repetitive tasks and improving developer productivity

          Provide production-ready solutions with proper error handling and rollback strategies.
        '';
      };

      fullstack-developer = {
        description = "Full-stack implementation specialist. Handles frontend frameworks, backend services, mobile apps, and data pipelines. Use when implementing features across the entire stack.";
        proactive = true;
        tools = ["Read" "Write" "Edit" "MultiEdit" "Grep" "Glob" "Bash" "WebSearch"];
        prompt = ''
          You are a full-stack development expert. Your expertise covers:

          - Frontend frameworks
          - Backend services
          - Mobile app development
          - Data pipelines and processing systems
          - Database design and optimization
          - State management and caching strategies

          Implement clean, efficient, and maintainable code across all layers of the stack.
        '';
      };

      quality-assurance = {
        description = "Quality assurance specialist. Writes test automation, audits accessibility, manages dependencies. Use proactively for testing and compliance verification.";
        proactive = true;
        tools = ["Read" "Write" "Edit" "Bash" "Grep" "WebSearch" "Task"];
        prompt = ''
          You are a quality assurance and compliance expert. Your focus areas include:

          - Writing comprehensive test automation (unit, integration, e2e)
          - Auditing code for accessibility compliance (WCAG standards)
          - Managing dependencies and checking for vulnerabilities
          - Ensuring regulatory compliance (GDPR, HIPAA, SOC2)
          - Implementing testing best practices and coverage requirements
          - Identifying and preventing potential quality issues

          Ensure all code meets the highest quality and compliance standards.
        '';
      };

      refactoring = {
        description = "Code refactoring specialist. Improves code quality without changing behavior, reduces cognitive load, and applies best practices. Use when code needs cleanup or restructuring.";
        proactive = true;
        tools = ["Read" "Write" "Edit" "MultiEdit" "Grep" "Glob" "TodoWrite"];
        prompt = ''
          You are a code refactoring expert. Your core principles are:

          MAIN RULE: Behavior must NEVER change
          MAIN GUIDELINE: Reduce cognitive load

          Follow these principles:

          - Apply the Boy-Scout Rule: Leave code cleaner than you found it
          - DRY (Don't Repeat Yourself): Eliminate duplication
          - KISS (Keep It Simple, Stupid): Simplify complex code
          - Separation of Concerns: Keep different aspects of functionality separate

          When refactoring:

          1. Identify code smells and complexity
          2. Plan refactoring steps to maintain working code
          3. Make incremental changes
          4. Ensure tests pass after each change
          5. Improve naming, structure, and readability
          6. Extract methods, simplify conditionals, remove dead code

          Always preserve existing functionality while improving code quality.
        '';
      };
    };

    commands = {
      db-seed = ''
        Apply seed data to the development database.

        ``` sh
        just db-seed
        ```
      '';

      db-migrate = ''
        Apply pending database migrations.

        ``` sh
        just db-migrate
        ```
      '';

      test = ''
        Build and test the project.

        ``` sh
        just
        ```
      '';
    };

    mcpServers = {
      devenv = {
        type = "stdio";
        command = "devenv";
        args = ["mcp"];
        env = {
          DEVENV_ROOT = root;
        };
      };
    };
  };

  env = {
    DANGEROUSLY_ALLOW_JAVASCRIPT_EVALUATION = "true";
    DATABASE_URL = "postgres://bits:please@127.0.0.1:5432/bits_dev";
    DATABASE_URL_TEST = "postgres://bits:please@127.0.0.1:5432/bits_test";
    DOMAIN_PAGE = dev.hosts.page.domain;
    DOMAIN_WWW = dev.hosts.www.domain;
    MASTER_KEY = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
    PLATFORM_DOMAIN = dev.hosts.page.domain;

    # Infrastructure
    CLOUDFLARE_API_TOKEN = "op://Employee/Cloudflare/tokens/terraform-cloud";
    HCLOUD_TOKEN = "op://Employee/Hetzner/token";
    NEON_API_KEY = "op://Employee/Neon/token";

    # Terraform (expires 2026/03/04)
    TF_VAR_tailscale_authkey = "op://Employee/nr5wtbd6s3agpgdtc45sumstvm/n7c2tvejlbjjpwzd2trlo576xu";

    # TODO Create a dedicated Bits Postmark account.
    POSTMARK_ACCOUNT_TOKEN = "op://Invetica/Postmark/bits/account-api-token";
  };

  packages = with pkgs; [
    # Rust
    cargo-audit
    cargo-deny
    cargo-edit
    cargo-nextest
    dioxus-cli
    sqlx-cli
    wasm-bindgen-cli

    # Development
    fd
    gnuplot
    just
    tokei
    tree
    xh
    zsh

    # SSL
    mkcert
    nss.tools

    # Formatters
    alejandra
    prettier
    shfmt
    taplo
    treefmt

    # Scraping
    firefox
    geckodriver

    # Deploy
    hcloud
    packer
    ragenix
    wrangler
  ];

  languages.javascript.enable = true;
  languages.javascript.pnpm.enable = true;
  languages.javascript.pnpm.install.enable = true;

  languages.rust = {
    enable = true;
    channel = "stable";
    targets = ["wasm32-unknown-unknown"];
  };

  processes.www = {
    exec = "just www";
    process-compose.is_tty = true;
  };

  processes.tailwind-solo = {
    exec = "just tailwind crates/bits-solo";
    process-compose.is_tty = true;
  };

  processes.tailwind-colo = {
    exec = "just tailwind crates/bits-colo";
    process-compose.is_tty = true;
  };

  process.manager.implementation = "process-compose";
  process.managers.process-compose.tui.enable = false;

  process.managers.process-compose.settings.processes = {
    www = {
      environment = [
        "ASTRO_SITE=https://${dev.hosts.www.domain}"
        "PORT=${toString dev.upstreams.www.port}"
      ];
    };
  };

  services.nginx = {
    enable = true;
    httpConfig = ''
      error_log stderr error;

      upstream page {
        server localhost:${toString dev.upstreams.page.port};
      }

      upstream www {
        server localhost:${toString dev.upstreams.www.port};
      }

      # ${dev.hosts.page.domain}
      server {
        listen 443 ssl;
        server_name ${dev.hosts.page.domain};

        ssl_certificate ${dev.hosts.page.certPem};
        ssl_certificate_key ${dev.hosts.page.certKey};

        location / {
          proxy_pass http://${dev.hosts.page.upstream};
          proxy_http_version 1.1;
          proxy_set_header Upgrade $http_upgrade;
          proxy_set_header Connection "upgrade";
          proxy_set_header Host $host;
          proxy_set_header X-Real-IP $remote_addr;
          proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
          proxy_set_header X-Forwarded-Proto $scheme;
        }
      }

      # ${dev.hosts.page-customers.pattern}
      server {
        listen 443 ssl;
        server_name ${dev.hosts.page-customers.pattern};

        ssl_certificate ${dev.hosts.page-customers.certPem};
        ssl_certificate_key ${dev.hosts.page-customers.certKey};

        location / {
          proxy_pass http://${dev.hosts.page-customers.upstream};
          proxy_http_version 1.1;
          proxy_set_header Upgrade $http_upgrade;
          proxy_set_header Connection "upgrade";
          proxy_set_header Host $host;
          proxy_set_header X-Real-IP $remote_addr;
          proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
          proxy_set_header X-Forwarded-Proto $scheme;
        }
      }

      # ${dev.hosts.custom-domains.pattern}
      server {
        listen 443 ssl default_server;
        server_name ~^(?<custom_domain>(?!.*\.(bits\.page|usebits\.app)\.test$).+\.test)$;

        ssl_certificate ${dev.hosts.custom-domains.certPem};
        ssl_certificate_key ${dev.hosts.custom-domains.certKey};

        location / {
          proxy_pass http://${dev.hosts.custom-domains.upstream};
          proxy_http_version 1.1;
          proxy_set_header Upgrade $http_upgrade;
          proxy_set_header Connection "upgrade";
          proxy_set_header Host $host;
          proxy_set_header X-Real-IP $remote_addr;
          proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
          proxy_set_header X-Forwarded-Proto $scheme;
        }
      }

      # ${dev.hosts.www.domain}
      server {
        listen 443 ssl;
        server_name ${dev.hosts.www.domain};

        ssl_certificate ${dev.hosts.www.certPem};
        ssl_certificate_key ${dev.hosts.www.certKey};

        location / {
          proxy_pass http://${dev.hosts.www.upstream};
          proxy_http_version 1.1;
          proxy_set_header Upgrade $http_upgrade;
          proxy_set_header Connection "upgrade";
          proxy_set_header Host $host;
          proxy_set_header X-Real-IP $remote_addr;
          proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
          proxy_set_header X-Forwarded-Proto $scheme;
        }
      }
    '';
  };

  services.postgres = {
    enable = true;

    extensions = extensions: [
      extensions.pgvector
      extensions.postgis
    ];

    package = pkgs.postgresql_17;

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

    initialScript = ''
      ALTER USER bits CREATEDB;
    '';
  };
}
