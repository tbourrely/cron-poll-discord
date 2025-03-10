run-bot:
	cargo watch -x run -- cargo run --bin bot

run-sender:
	cargo watch -x run -- cargo run --bin sender

run-api:
	cargo watch -x run -- cargo run --bin api

test:
	cargo test

delete-db:
	rm -f polls.sqlite

integration-tests:
	rm -f venom.*
	venom run venom/
