# LIBCOSMIC

Building blocks for COSMIC applications.

## Building
Libcosmic is written in pure Rust, so `cargo` is all you need.

```shell
cargo build
```

## Usage
There's examples in the `examples` directory.

### Widget library
```shell
cargo run --release --example cosmic
```

On Pop!_OS
```shell
sudo apt install cargo libexpat1-dev libfontconfig-dev libfreetype-dev pkg-config cmake
git clone https://github.com/pop-os/libcosmic
cd libcosmic
git submodule update --init
cargo run --release -p cosmic
```

### Text rendering
```shell
cargo run --release --example text
```

## Documentation
The documentation can be found [here](https://pop-os.github.io/docs/).

## Licence
Libcosmic is licenced under the MPL-2.0

## Contact
- [Mattermost](https://chat.pop-os.org/)
- [Discord](https://chat.pop-os.org/)
- [Twitter](https://twitter.com/pop_os_official)
- [Instagram](https://www.instagram.com/pop_os_official/)
