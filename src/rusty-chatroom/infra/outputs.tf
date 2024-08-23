output "kv_binding_id" {
  value = cloudflare_workers_kv_namespace.rusty_serverless.id
}

output "kv_binding_name" {
  value = cloudflare_workers_kv_namespace.rusty_serverless.title
}