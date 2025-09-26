{pkgs, ...}: {
  env = {
    FASTLY_DISABLE_WASM_BUILD_TELEMETRY = "1";
  };

  packages = with pkgs; [
    cargo-deny
    cargo-nextest
    cargo-watch
  ];

  languages.rust = {
    enable = true;
    channel = "stable";
    targets = ["wasm32-wasip1"];
  };
}
