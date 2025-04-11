run-bot:
	cargo watch -x run -- cargo run --bin bot

run-sender:
	cargo watch -x run -- cargo run --bin sender

run-api:
	cargo watch -x run -- cargo run --bin api

test:
	cargo test

dev-start:
	docker compose --profile dev up

integration-tests:
	rm -f venom.*
	venom run --stop-on-failure venom/
