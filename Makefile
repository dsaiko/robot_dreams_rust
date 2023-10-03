build: clean codecheck fmt fix doc test
	cargo build
	cargo build --release

clean:
	cargo clean

codecheck:
	cargo clippy --tests

test:
	cargo test

fmt:
	cargo fmt

fix:
	cargo fix --allow-dirty

doc:
	cargo doc