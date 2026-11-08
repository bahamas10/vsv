.PHONY: all
all:
	@echo 'nothing to do'

.PHONY: man
man: man/vsv.8
man/vsv.8: man/vsv.md
	md2man-roff $^ > $@

.PHONY: clean
clean:
	rm -f man/vsv.8

.PHONY: check
check:
	cargo fmt --check
	cargo check --all-targets --all-features
	cargo clippy --all-targets --all-features -- -D warnings

.PHONY: fmt
fmt:
	cargo fmt

.PHONY: test
test:
	cargo test --all-targets --all-features

.PHONY: test-basic
test-basic:
	cargo test --test basic

.PHONY: test-integration
test-integration:
	cargo test --test integration
