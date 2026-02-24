{pkgs}: let
  jdk = pkgs.temurin-bin-25;
in {
  inherit jdk;

  packages = with pkgs; [
    # Clojure
    clj-kondo
    (clojure.override {inherit jdk;})
    jdk

    # Development
    just
    tailwindcss_4

    # Formatters
    alejandra
    cljfmt
    prettier
    shfmt
    taplo
    treefmt
  ];
}
