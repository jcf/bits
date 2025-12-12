final: prev: {
  dioxus-cli = final.rustPlatform.buildRustPackage rec {
    pname = "dioxus-cli";
    version = "0.7.2";

    src = final.fetchCrate {
      inherit pname version;
      hash = "sha256-VCoTxZKFYkGBCu1X/9US/OCFpp6zc5ojmXWJfzozCxc=";
    };

    cargoHash = "sha256-de8z68uXnrzyxTJY53saJ6hT7rvYbSdsSA/WWQa6nl4=";

    nativeBuildInputs = with final; [pkg-config];

    buildInputs = with final;
      [openssl]
      ++ lib.optionals stdenv.hostPlatform.isDarwin [
        apple-sdk_15
      ];

    doCheck = false;
  };
}
