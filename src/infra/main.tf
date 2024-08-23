resource "cloudflare_workers_kv_namespace" "rusty_serverless" {
  account_id = var.cloudflare_account_id
  title      = "rusty-serverless"
}

resource "cloudflare_d1_database" "rusty_serverless_chat_metadata" {
  account_id = var.cloudflare_account_id
  name       = "rusty-serverless-chat-metadata"
}