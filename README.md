# Cloudflare Workers + Rust

Sample repository playing around with Rust + CloudFlare Workers. Currently, there are 2 examples:

## Axum + Postgres

[LINK](./src/axum-postgres/)

An example of using the Axum web framework inside a CloudFlare worker, connecting to a Postgres databases.

## Router + KV

[LINK](./src/router-kv/)

A CloudFlare native approach, using the `worker::Router` for web routing and KV as a very simple key-value database.