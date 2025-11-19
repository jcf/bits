final: prev: {
  wasm-bindgen-cli = final.rustPlatform.buildRustPackage rec {
    pname = "wasm-bindgen-cli";

    # Version required by Dioxus.
    version = "0.2.105";

    src = final.fetchCrate {
      inherit pname version;
      hash = "sha256-zLPFFgnqAWq5R2KkaTGAYqVQswfBEYm9x3OPjx8DJRY=";
    };

    cargoHash = "sha256-a2X9bzwnMWNt0fTf30qAiJ4noal/ET1jEtf5fBFj5OU=";
    nativeBuildInputs = [final.pkg-config];
    buildInputs = [final.openssl];
    checkInputs = [final.nodejs];
    doCheck = false;
  };
}
