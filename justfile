# Run tests
test *args:
    cargo test {{args}} -- --nocapture

# Run clippy
clippy:
    cargo clippy -- -D warnings

# Check compilation
check:
    cargo check

# Build release binary
build:
    cargo build --release

# Install globally (incremental — fast after code changes)
install:
    cargo install --path .

# Run drill with arguments
run *args:
    cargo run -- {{args}}
