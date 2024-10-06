resource "cloudflare_workers_kv_namespace" "rusty_serverless" {
  account_id = var.cloudflare_account_id
  title      = "rusty-serverless"
}

resource "cloudflare_d1_database" "rusty_serverless_chat_metadata" {
  account_id = var.cloudflare_account_id
  name       = "rusty-serverless-chat-metadata"
}

resource "cloudflare_hyperdrive_config" "users_db" {
  account_id = var.cloudflare_account_id
  name       = "account-db"
  origin = {
    database = var.db_name
    password = var.db_password
    host     = var.db_host
    port     = var.db_port
    scheme   = "postgres"
    user     = var.db_user
  }
}

resource "cloudflare_queue" "user_notification_queue" {
  account_id = var.cloudflare_account_id
  name       = "user-notifications"
}