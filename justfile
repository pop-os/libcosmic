examples := 'applet application calendar config context-menu cosmic image-button menu multi-window nav-context open-dialog table-view'
clippy_args := '-W clippy::all -W clippy::pedantic'

# Check for errors and linter warnings
check *args: (check-wayland args) (check-winit args) (check-examples args)

check-examples *args:
    #!/bin/bash
    for project in {{examples}}; do
        cargo clippy -p ${project} {{args}} -- {{clippy_args}}
    done

check-wayland *args:
    cargo clippy --no-deps --features="wayland,tokio,xdg-portal" {{args}} -- {{clippy_args}}

check-winit *args:
    cargo clippy --no-deps --features="winit,tokio,xdg-portal" {{args}} -- {{clippy_args}}

# Runs a check with JSON message format for IDE integration
check-json: (check '--message-format=json')

# Remove Cargo build artifacts
clean:
    cargo clean

# Also remove .cargo and vendored dependencies
clean-dist: clean
    rm -rf .cargo vendor vendor.tar target

# Runs an example of the given {{name}}
run name:
    cargo run --release -p {{name}}
