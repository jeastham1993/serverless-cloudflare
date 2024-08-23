output "kv_binding_id" {
  value = cloudflare_workers_kv_namespace.rusty_serverless.id
}

output "kv_binding_name" {
  value = cloudflare_workers_kv_namespace.rusty_serverless.title
}

output "d1_database_id" {
  value = cloudflare_d1_database.rusty_serverless_chat_metadata.id
}

output "d1_database_name" {
  value = cloudflare_d1_database.rusty_serverless_chat_metadata.name
}