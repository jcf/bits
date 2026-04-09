{
  container-base,
  nix2container,
  pkgs,
}: let
  inherit (pkgs) bash buildEnv;

  # Use the wrapped tailwindcss binary (needs bash for the wrapper script
  # that sets LD_LIBRARY_PATH for the lightningcss native module).
  tailwindcss = pkgs.tailwindcss_4;

  rootLayer = buildEnv {
    name = "tailwind-root";
    paths =
      container-base.paths
      ++ [bash tailwindcss];
  };
in
  nix2container.buildImage {
    name = "bits-tailwind";

    copyToRoot = [rootLayer];

    config = {
      Labels = container-base.labels "bits-tailwind" "Tailwind CSS watcher";

      Entrypoint = ["/bin/tailwindcss"];

      Env = [
        "PATH=/bin"
      ];

      User = "${container-base.uid}:${container-base.uid}";
      WorkingDir = "/app";
    };
  }
