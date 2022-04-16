test:
	cargo test --lib -p c3-lang-linearization
	cargo test --lib -p c3-lang-parser

lint:
	cargo fmt
	cargo clippy --all-targets -- -D warnings \
		-A clippy::new-without-default \
		-A clippy::needless-lifetimes \
		-A clippy::clone-on-copy \
		-A clippy::just-underscores-and-digits
