wasm-target:
	rustup target add wasm32-unknown-unknown

dev-pages:
	cd src;npx wrangler pages dev

dev-backend:
	cd src/backend;npx wrangler dev

test:
	cd src/backend;npx wrangler deploy --dry-run;cd ../../;npm run test

db-migrations-local:
	cd src/backend;npx wrangler d1 migrations apply rusty-serverless-chat-metadata

db-migrations:
	cd src/backend;npx wrangler d1 migrations apply rusty-serverless-chat-metadata --remote