{
  config,
  pkgs,
  ...
}: {
  claude.code = {
    enable = true;

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

  env = {
    CLOUDFLARE_API_TOKEN = "op://Employee/Cloudflare/tokens/terraform-cloud";
  };

  packages = with pkgs; [
    # Development
    fd
    just
    zsh

    # Formatters
    alejandra
    prettier
    shfmt
    taplo
    treefmt
  ];

  languages.javascript.enable = true;
  languages.javascript.pnpm.enable = true;

  process.manager.implementation = "process-compose";
  process.managers.process-compose.tui.enable = false;
}
