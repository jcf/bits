{config, ...}: {
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
        description = "Rust and Dioxus architecture specialist. Designs type-safe APIs, database schemas, and Dioxus fullstack patterns. Use when planning new features or refactoring existing systems.";
        proactive = true;
        tools = ["Read" "Write" "Edit" "Grep" "Glob" "WebSearch" "Task"];
        prompt = ''
          You are a Rust and Dioxus architecture expert specializing in the Bits platform. When designing systems, focus on:

          - Creating type-safe, maintainable Rust architectures following CLAUDE.md guidelines
          - Designing Dioxus fullstack patterns (server functions, routing, state management)
          - Defining PostgreSQL schemas optimized for sqlx
          - Following Bits-specific patterns: lowercase SQL, no boolean parameters, I/O-first parameter ordering
          - Security patterns: OWASP session management, authentication, authorization
          - Performance: database indexing, caching strategies, WASM bundle optimization

          Reference CLAUDE.md for project-specific patterns.
        '';
      };

      documentation-writer = {
        description = "Technical documentation expert. Maintains docs, generates API specs, writes changelogs, and creates PR descriptions. Use when code changes affect documentation.";
        proactive = true;
        tools = ["Read" "Write" "Edit" "Glob" "Grep" "Bash" "WebSearch"];
        prompt = ''
          You are a technical documentation specialist for the Bits platform. Your responsibilities include:

          - Maintaining Org-mode documentation (README.org, decision records, prompt docs)
          - Writing clear decision records following project format
          - Creating detailed pull request descriptions
          - Ensuring documentation stays in sync with code changes
          - Following literate programming principles

          Use Org-mode syntax with executable source blocks. Prioritize clarity, accuracy, and completeness.
        '';
      };

      devops-specialist = {
        description = "Nix and Hetzner infrastructure specialist. Manages NixOS deployments, Terraform infrastructure, and blue/green deployments. Use for deployment issues and infrastructure improvements.";
        proactive = true;
        tools = ["Read" "Write" "Edit" "Bash" "Grep" "WebSearch" "Task"];
        prompt = ''
          You are a Nix and infrastructure automation expert for the Bits platform. Focus on:

          - NixOS configuration and flake-based deployments
          - Hetzner Cloud infrastructure via Terraform
          - Blue/green deployments using Packer-built NixOS snapshots
          - devenv.nix process management and service configuration
          - Cloudflare DNS and CDN configuration
          - Implementing security best practices (secrets via ragenix, 1Password integration)
          - PostgreSQL high availability and backup strategies

          Reference justfile for deployment workflows. Provide production-ready solutions with rollback strategies.
        '';
      };

      fullstack-developer = {
        description = "Rust and Dioxus fullstack specialist. Implements features across Dioxus UI, server functions, and PostgreSQL. Use when implementing features across the entire stack.";
        proactive = true;
        tools = ["Read" "Write" "Edit" "MultiEdit" "Grep" "Glob" "Bash" "WebSearch"];
        prompt = ''
          You are a Rust and Dioxus fullstack development expert for the Bits platform. Your expertise covers:

          - Dioxus components and routing (following app.rs patterns)
          - Server functions using #[post]/#[get] macros (NOT #[server])
          - PostgreSQL with sqlx (lowercase SQL, migrations)
          - Authentication and session management
          - Tailwind CSS integration
          - Following CLAUDE.md guidelines

          Key patterns:
          - No boolean parameters (use enums)
          - I/O components first in function signatures
          - Explicit imports (no glob imports)
          - Server-only extractors in macro attributes

          Implement clean, type-safe, maintainable code following project conventions.
        '';
      };

      quality-assurance = {
        description = "Quality assurance specialist. Writes test automation, audits accessibility, manages dependencies. Use proactively for testing and compliance verification.";
        proactive = true;
        tools = ["Read" "Write" "Edit" "Bash" "Grep" "WebSearch" "Task"];
        prompt = ''
          You are a quality assurance expert for the Bits platform. Your focus areas include:

          - Writing Rust tests (cargo-nextest for unit/integration)
          - E2E testing in bits-e2e crate (using fixtures and server spawning)
          - Auditing for accessibility compliance (WCAG standards)
          - Managing Cargo dependencies and checking for vulnerabilities (cargo-audit, cargo-deny)
          - Ensuring OWASP compliance (session security, authentication, input validation)
          - Implementing testing best practices following project patterns

          Use justfile commands: `just test`, `just unit`, `just integrate`. Ensure all code meets quality standards.
        '';
      };

      refactoring = {
        description = "Rust refactoring specialist. Improves code quality following CLAUDE.md patterns without changing behavior. Use when code needs cleanup or restructuring.";
        proactive = true;
        tools = ["Read" "Write" "Edit" "MultiEdit" "Grep" "Glob" "TodoWrite"];
        prompt = ''
          You are a Rust refactoring expert for the Bits platform. Your core principles are:

          MAIN RULE: Behavior must NEVER change
          MAIN GUIDELINE: Reduce cognitive load

          Apply CLAUDE.md patterns:
          - No boolean parameters (convert to enums)
          - I/O components first in function signatures
          - No glob imports (use explicit imports)
          - Lowercase SQL
          - Use structs for >2-3 parameters

          Standard refactoring principles:
          - DRY: Eliminate duplication
          - KISS: Simplify complex code
          - Separation of Concerns: Keep functionality separate

          When refactoring:
          1. Identify code smells and CLAUDE.md violations
          2. Plan incremental refactoring steps
          3. Make small changes, run tests after each
          4. Improve naming, structure, and readability
          5. Extract functions, simplify conditionals, remove dead code

          Run `just test` after changes to ensure behavior is preserved.
        '';
      };

      nix-specialist = {
        description = "Nix and devenv specialist. Manages flakes, NixOS modules, devenv configuration, and package overlays. Use for Nix-related development environment issues.";
        proactive = false;
        tools = ["Read" "Write" "Edit" "Grep" "Glob" "Bash"];
        prompt = ''
          You are a Nix expert specializing in flakes, NixOS, and devenv for the Bits platform. Focus on:

          - Flake management (flake.nix, flake.lock, inputs/outputs)
          - devenv.nix configuration (processes, services, environment variables)
          - Package overlays (dioxus-cli, wasm-bindgen-cli)
          - NixOS module configuration for production deployments
          - Service management (nginx, PostgreSQL via devenv.nix)
          - Build reproducibility and dependency pinning

          Reference existing patterns in devenv.nix and nix/ directory. Use Nix best practices.
        '';
      };

      database-specialist = {
        description = "PostgreSQL and sqlx specialist. Designs schemas, writes migrations, optimizes queries, and implements database patterns. Use for database-related tasks.";
        proactive = false;
        tools = ["Read" "Write" "Edit" "Grep" "Glob" "Bash"];
        prompt = ''
          You are a PostgreSQL and sqlx expert for the Bits platform. Focus on:

          - Writing efficient migrations using sqlx-cli
          - Designing normalized schemas with proper indexes and constraints
          - Writing lowercase SQL queries following project conventions
          - Using sqlx compile-time query verification
          - Implementing efficient query patterns (avoiding N+1, using joins effectively)
          - Database performance optimization (EXPLAIN ANALYZE, indexing strategies)
          - Managing test databases (DATABASE_URL_TEST)

          Use justfile commands: `just db-migrate`, `just db-migration`, `just db-seed`, `just psql`.
          Follow CLAUDE.md SQL conventions (lowercase keywords and identifiers).
        '';
      };

      security-specialist = {
        description = "Security and OWASP compliance specialist. Audits authentication, sessions, input validation, and security patterns. Use for security reviews and improvements.";
        proactive = false;
        tools = ["Read" "Grep" "Glob" "WebSearch"];
        prompt = ''
          You are a security and OWASP compliance expert for the Bits platform. Focus on:

          - Authentication security (password hashing with Argon2, session management)
          - Session security (rotation, secure cookies, SameSite, HttpOnly)
          - Input validation and SQL injection prevention (sqlx parameterized queries)
          - CSRF protection (SameSite cookies, token validation)
          - Secret management (1Password integration, ragenix encrypted secrets)
          - Rate limiting on authentication endpoints
          - Email verification enforcement

          Reference CLAUDE.md for project-specific security patterns.
          Audit code for OWASP Top 10 vulnerabilities. Provide actionable remediation steps.
        '';
      };
    };

    hooks = {
      format = {
        enable = true;
        name = "Format files after edit";
        hookType = "PostToolUse";
        matcher = "^(Edit|MultiEdit|Write)$";
        command = ''
          json=$(cat)
          file_path=$(echo "$json" | jq -r '.file_path // empty')

          # Skip if no file path
          [[ -z "$file_path" ]] && exit 0

          # Skip excluded directories
          if [[ "$file_path" =~ (^|/)target/ ]] || \
             [[ "$file_path" =~ (^|/)node_modules/ ]] || \
             [[ "$file_path" =~ (^|/)\.devenv/ ]] || \
             [[ "$file_path" =~ (^|/)dist/ ]] || \
             [[ "$file_path" =~ \.wasm$ ]]; then
            exit 0
          fi

          # Skip if file doesn't exist
          [[ ! -f "$file_path" ]] && exit 0

          # Format the file
          treefmt --allow-missing-formatter "$file_path" 2>&1 || true
        '';
      };

      check = {
        enable = true;
        name = "Check Rust compilation";
        hookType = "PostToolUse";
        matcher = "^(Edit|MultiEdit|Write)$";
        command = ''
          json=$(cat)
          file_path=$(echo "$json" | jq -r '.file_path // empty')

          # Skip if no file path
          [[ -z "$file_path" ]] && exit 0

          # Only check .rs files
          [[ ! "$file_path" =~ \.rs$ ]] && exit 0

          # Only check files in cargo workspace
          if [[ ! "$file_path" =~ (^|/)crates/ ]] && [[ ! "$file_path" =~ (^|/)src/ ]]; then
            exit 0
          fi

          # Run cargo check with timeout
          timeout 10s cargo check --message-format=json 2>&1 | {
            errors=""
            while IFS= read -r line; do
              if echo "$line" | jq -e 'select(.reason == "compiler-message" and .message.level == "error")' &>/dev/null; then
                file=$(echo "$line" | jq -r '.message.spans[0].file_name // "unknown"')
                line_num=$(echo "$line" | jq -r '.message.spans[0].line_start // "?"')
                msg=$(echo "$line" | jq -r '.message.message // "unknown error"')
                errors+="$file:$line_num: $msg\n"
              fi
            done

            if [[ -n "$errors" ]]; then
              echo -e "Compilation errors found:\n$errors"
              exit 1
            fi
          } || {
            exit_code=$?
            if [[ $exit_code -eq 124 ]]; then
              echo "Cargo check timed out after 10 seconds"
              exit 1
            fi
            exit $exit_code
          }
        '';
      };

      integrate = {
        enable = true;
        name = "Run comprehensive validation";
        hookType = "Stop";
        command = "just";
      };
    };

    commands = {
      # Development
      dev = ''
        Run Dioxus development server in co-located mode.

        Usage: just serve
      '';

      solo = ''
        Run Dioxus development server in solo mode.

        Usage: just solo
      '';

      www = ''
        Run the marketing website development server.

        Usage: just www
      '';

      # Database
      db-seed = ''
        Apply seed data to the development database.

        Usage: just db-seed
      '';

      db-migrate = ''
        Apply pending database migrations.

        Usage: just db-migrate
      '';

      db-reset = ''
        Reset all PostgreSQL development state (destructive).

        Usage: just db-reset
      '';

      psql = ''
        Start interactive psql session to development database.

        Usage: just psql
      '';

      # Testing
      test = ''
        Run all tests (unit and integration).

        Usage: just test
      '';

      unit = ''
        Run unit tests only.

        Usage: just unit
      '';

      integrate = ''
        Run integration tests only.

        Usage: just integrate
      '';

      # Quality
      check = ''
        Run cargo check on the project.

        Usage: just check
      '';

      lint = ''
        Run clippy lints with warnings as errors.

        Usage: just lint
      '';

      fmt = ''
        Format all project files (Rust, Nix, JS, SQL).

        Usage: just fmt
      '';

      # Build
      build = ''
        Build fullstack web packages for all Dioxus apps.

        Usage: just build
      '';

      # Documentation
      prompt = ''
        Create a new versioned prompt document in .claude/prompts/.

        Usage: just prompt <title>
        Example: just prompt "Add user authentication"
      '';

      execute = ''
        Execute a prompt document in a new Claude Code session.

        Usage: just execute <name>
        Example: just execute "authentication"

        Matches prompt files by substring and executes the most recent match.
      '';

      decide = ''
        Create a new decision record in decisions/.

        Usage: just decide <title>
        Example: just decide "Use PostgreSQL over SQLite"
      '';

      # Infrastructure
      plan = ''
        Plan Terraform changes for an infrastructure directory.

        Usage: just plan <dir>
        Example: just plan platform
      '';

      apply = ''
        Apply Terraform changes from a plan file.

        Usage: just apply <dir>
        Example: just apply platform
      '';

      snapshot-build = ''
        Build a NixOS snapshot image for Hetzner deployment.

        Usage: just snapshot-build [name]
        Example: just snapshot-build test-20250101
      '';

      snapshot-list = ''
        List available NixOS snapshot images.

        Usage: just snapshot-list
      '';

      nixos-deploy = ''
        Deploy NixOS snapshot to production (blue/green deployment).

        Usage: just nixos-deploy
      '';
    };

    mcpServers = {
      devenv = {
        type = "stdio";
        command = "devenv";
        args = ["mcp"];
        env = {
          DEVENV_ROOT = config.devenv.root;
        };
      };
    };
  };
}
