let
  users = {
    jcf = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIJCFqBB/awADSj49zwaFnni4O1KHedQTax4b/8RyvMfX";
  };
in {
  # Development secrets
  "database_url_dev.age".publicKeys = [users.jcf];
  "master_key_dev.age".publicKeys = [users.jcf];

  # Production secrets
  "database_url_prod.age".publicKeys = [users.jcf];
  "master_key_prod.age".publicKeys = [users.jcf];
}
