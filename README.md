# Cloudflare Workers + Rust

Sample repository playing around with Rust + CloudFlare Workers. Currently, there are 2 examples:

## Axum + Postgres

[LINK](./src/axum-postgres/)

An example of using the Axum web framework inside a CloudFlare worker, connecting to a Postgres databases. To use this example, you will need to set 4 environment variables against your CloudFlare worker:

- DB_HOST
- DB_NAME
- DB_USER
- DB_PASS

## Router + KV

[LINK](./src/router-kv/)

A CloudFlare native approach, using the `worker::Router` for web routing and KV as a very simple key-value database.