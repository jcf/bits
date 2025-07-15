{pkgs, ...}: {
  languages.javascript.enable = true;
  languages.javascript.pnpm.enable = true;
  languages.javascript.pnpm.install.enable = true;

  process.manager.implementation = "process-compose";
  process.managers.process-compose.tui.enable = false;

  services.postgres = {
    enable = true;

    package = pkgs.postgresql_16;

    listen_addresses = "127.0.0.1";
    initialDatabases = [
      {
        name = "bits_dev";
        user = "bits";
        pass = "please";
      }
      {
        name = "bits_test";
        user = "bits";
        pass = "please";
      }
    ];
  };
}
