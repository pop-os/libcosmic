projects := 'application cosmic cosmic_sctk design open_dialog'

# Check for errors and linter warnings
check *args:
    cargo clippy --no-deps {{args}} -- -W clippy::pedantic
    cargo clippy --no-deps --no-default-features --features="winit,tokio" {{args}} -- -W clippy::pedantic
    for project in {{projects}}; do \
        cargo check -p ${project}; \
    done

# Runs a check with JSON message format for IDE integration
check-json: (check '--message-format=json')

# Runs an example of the given {{name}}
example name:
    cargo run --release -p {{name}}