run:
	cargo run

run-watch:
	cargo-watch -x run --ignore *.sqlite

test:
	cargo test

delete-db:
	rm -f polls.sqlite

clean-run: delete-db run
