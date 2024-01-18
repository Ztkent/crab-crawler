.PHONY: build run clean test update

build:
	@cargo build

run:
	@cargo run

clean:
	@cargo clean

test:
	@cargo test

update:
	@cargo update