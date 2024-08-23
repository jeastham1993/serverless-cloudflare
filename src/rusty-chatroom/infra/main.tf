resource "cloudflare_workers_kv_namespace" "rusty_serverless" {
  account_id = var.cloudflare_account_id
  title      = "rusty-serverless"
}