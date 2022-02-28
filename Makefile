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
