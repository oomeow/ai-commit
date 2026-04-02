build:
    cargo build -r
    cp target/release/ai-commit ~/.bin/ai-commit

commit:
    git add .
    ai-commit
