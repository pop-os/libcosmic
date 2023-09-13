# Use mold linker if clang and mold exists.
clang-path := `which clang || true`
mold-path := `which mold || true`

linker-arg := if clang-path != '' {
    if mold-path != '' {
        '-C linker=' + clang-path + ' -C link-arg=--ld-path=' + mold-path + ' '
    } else {
        ''
    }
} else {
    ''
}

export RUSTFLAGS := linker-arg + env_var_or_default('RUSTFLAGS', '')

[private]
default: build-release

# Compile with release profile
build-release *args: (build '--release' args)

# Compile with a vendored tarball
build-vendored *args: vendor-extract (build-release '--offline --locked' args)

# Vendor Cargo dependencies locally
vendor *args:
    rm -rf .cargo vendor vendor.tar
    mkdir -p .cargo
    cargo vendor {{args}} | head -n -1 > .cargo/config
    echo 'directory = "vendor"' >> .cargo/config
    tar pcf vendor.tar vendor
    rm -rf vendor

# Extracts vendored dependencies
[private]
vendor-extract:
    rm -rf vendor
    tar pxf vendor.tar
