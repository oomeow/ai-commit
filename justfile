set windows-shell := ["nu", "-c"]

dev *command:
    RUST_LOG=debug,prek=off cargo run -- -f ./config.dev.toml {{command}}

test *command:
    cargo run -- -f ./config.dev.toml {{command}}

completion:
    cargo run -- completion zsh > ./completions/zsh/_ai-commit
    cargo run -- completion bash > ./completions/bash/_ai-commit
    cargo run -- completion fish > ./completions/fish/ai-commit.fish

build:
    cargo build -r
    # cp target/release/ai-commit ~/.bin/ai-commit

commit:
    git add .
    ai-commit
