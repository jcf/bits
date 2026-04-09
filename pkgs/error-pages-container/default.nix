{
  container-base,
  error-pages-dir,
  fonts-dir,
  nix2container,
  pkgs,
}: let
  files = pkgs.runCommand "error-pages-files" {} ''
    mkdir -p $out/app/bin $out/www/fonts
    cp ${pkgs.darkhttpd}/bin/darkhttpd $out/app/bin/
    cp ${error-pages-dir}/502.html $out/www/
    cp ${fonts-dir}/DMSans.woff2 $out/www/fonts/ 2>/dev/null || true
    cp ${fonts-dir}/JetBrainsMono.woff2 $out/www/fonts/ 2>/dev/null || true
  '';

  rootLayer = pkgs.buildEnv {
    name = "error-pages-root";
    paths = container-base.paths ++ [files];
  };
in
  nix2container.buildImage {
    name = "bits-error-pages";

    copyToRoot = [rootLayer];

    config = {
      Labels = container-base.labels "bits-error-pages" "Error pages for Traefik";

      Entrypoint = ["/app/bin/darkhttpd" "/www" "--port" "8080" "--no-listing"];

      Env = [
        "LD_LIBRARY_PATH=/lib"
      ];

      ExposedPorts."8080/tcp" = {};
      User = "${container-base.uid}:${container-base.uid}";
      WorkingDir = "/www";
    };
  }
