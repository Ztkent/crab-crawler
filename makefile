.PHONY: build run clean test update release

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

release:
	@cargo build --release