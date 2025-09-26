{pkgs, ...}: {
  claude.code.enable = true;

  packages = with pkgs; [
    # Development
    just

    # Formatting
    alejandra
    prettier
    shfmt
    taplo
    treefmt
  ];

  languages.javascript.enable = true;
  languages.javascript.pnpm.enable = true;

  processes.edit.exec = "pnpm dev:edit";
  processes.page.exec = "pnpm dev:page";

  process.managers.process-compose.settings.processes = {
    edit = {
      environment = [
        "PORT=3000"
      ];
    };
    page = {
      environment = [
        "PORT=3030"
      ];
    };
  };
}
