terraform {
  backend "pg" {
    conn_str = "postgres://terraform@compute:5432/terraform?sslmode=verify-full"
  }
}
