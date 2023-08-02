# Check for errors and linter warnings
check *args:
    cargo clippy --no-deps {{args}} -- -W clippy::pedantic
    cargo clippy --no-deps --no-default-features --features="winit,tokio" {{args}} -- -W clippy::pedantic
    cargo check -p application {{args}}
    cargo check -p cosmic {{args}}
    cargo check -p cosmic_sctk {{args}}

# Runs a check with JSON message format for IDE integration
check-json: (check '--message-format=json')

# Runs an example of the given {{name}}
example name:
    cargo run --release -p {{name}}