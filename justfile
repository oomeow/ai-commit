dev *command:
    RUST_LOG=debug cargo run -- {{command}}

test *command:
    cargo run -- {{command}}

build:
    cargo build -r
    cp target/release/ai-commit ~/.bin/ai-commit

commit:
    git add .
    ai-commit
