.PHONY: check
check:
	cargo check
	cargo clippy

.PHONY: fmt
fmt:
	cargo fmt

.PHONY: test
test:
	cargo test

.PHONY: test-basic
test-basic:
	cargo test --test basic

.PHONY: test-integration
test-integration:
	cargo test --test integration
