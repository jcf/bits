{config, ...}: let
  root = config.devenv.root;
in {
  claude.code = {
    enable = true;

    commands = {
      test = ''
        Run Kaocha tests

        ```bash
        just test
        ```
      '';

      lint = ''
        Run clj-kondo linting

        ```bash
        just lint
        ```
      '';

      fmt = ''
        Format all project files

        ```bash
        just fmt
        ```
      '';

      check = ''
        Run all quality checks (format, lint, test)

        ```bash
        just
        ```
      '';

      migration = ''
        Create a new database migration

        ```bash
        just migration <name>
        ```
      '';

      psql = ''
        Start an interactive psql session

        ```bash
        just psql
        ```
      '';
    };

    agents = {
      spec-reviewer = {
        description = "Reviews Clojure specs for correctness and consistency";
        proactive = true;
        tools = ["Read" "Grep"];
        prompt = ''
          You review Clojure specs in bits.spec. Check for:
          - Specs defined in bits.spec to avoid cyclic dependencies
          - Use of literal keywords when specs can't require component namespaces
          - Proper spec composition with s/keys, s/and, s/or
          - Consistency with existing spec patterns in the codebase
        '';
      };

      component-reviewer = {
        description = "Reviews Component lifecycle implementations";
        proactive = true;
        tools = ["Read" "Grep"];
        prompt = ''
          You review Clojure Component implementations. Check for:
          - API functions first, then record, then factory, then print method
          - Factory named make-<component> with :pre validation
          - No defaults in component - all defaults in bits.app/read-config
          - Print method hides sensitive data (hashes, connections, secrets)
          - I/O component as first parameter in API functions
        '';
      };

      test-runner = {
        description = "Runs Kaocha tests after code changes";
        proactive = false;
        tools = ["Bash"];
        prompt = ''
          Run the test suite using `just test`.
          Report any failures clearly with file paths and line numbers.
          If tests pass, confirm success concisely.
        '';
      };
    };

    hooks = {
      format-on-edit = {
        enable = true;
        name = "Format edited Clojure files";
        hookType = "PostToolUse";
        matcher = "^(Edit|Write)$";
        command = ''
          file_path=$(jq -r '.tool_input.file_path // empty')
          if [[ -n "$file_path" && -f "$file_path" ]]; then
            case "$file_path" in
              *.clj|*.cljc|*.cljs|*.edn)
                cljfmt fix "$file_path" 2>/dev/null || true
                ;;
            esac
          fi
        '';
      };

      lint-on-edit = {
        enable = true;
        name = "Lint edited Clojure files";
        hookType = "PostToolUse";
        matcher = "^(Edit|Write)$";
        command = ''
          file_path=$(jq -r '.tool_input.file_path // empty')
          if [[ -n "$file_path" && -f "$file_path" ]]; then
            case "$file_path" in
              *.clj|*.cljc|*.cljs)
                result=$(clj-kondo --lint "$file_path" 2>&1)
                exit_code=$?
                if [[ $exit_code -ne 0 ]]; then
                  echo "$result"
                  exit $exit_code
                fi
                ;;
            esac
          fi
        '';
      };
    };

    mcpServers = {
      clojure-mcp = {
        type = "stdio";
        command = "clojure";
        args = ["-Tmcp" "start" ":port" "9999" ":config-profile" ":cli-assist"];
      };
      devenv = {
        type = "stdio";
        command = "devenv";
        args = ["mcp"];
        env.DEVENV_ROOT = root;
      };
      github = {
        type = "http";
        url = "https://api.githubcopilot.com/mcp/";
      };
      linear = {
        type = "http";
        url = "https://mcp.linear.app/mcp";
      };
    };
  };
}
