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
}
