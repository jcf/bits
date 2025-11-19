final: prev: {
  dioxus-cli = final.rustPlatform.buildRustPackage rec {
    pname = "dioxus-cli";
    version = "0.7.1";

    src = final.fetchCrate {
      inherit pname version;
      hash = "sha256-tPymoJJvz64G8QObLkiVhnW0pBV/ABskMdq7g7o9f1A=";
    };

    cargoHash = "sha256-mgscu6mJWinB8WXLnLNq/JQnRpHRJKMQXnMwECz1vwc=";

    nativeBuildInputs = with final; [pkg-config];

    buildInputs = with final;
      [openssl]
      ++ lib.optionals stdenv.hostPlatform.isDarwin [
        apple-sdk_15
      ];

    doCheck = false;
  };
}
