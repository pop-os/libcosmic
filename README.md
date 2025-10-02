# LIBCOSMIC

A platform toolkit based on iced for creating applets and applications for the COSMIC™ desktop.

## Documentation

- [API Documentation](https://pop-os.github.io/libcosmic/cosmic/): Automatically generated from this repository via `cargo doc`
- [libcosmic Book](https://pop-os.github.io/libcosmic-book/): A reference for learning libcosmic

## Templates

- https://github.com/pop-os/cosmic-app-template: Application project template

## Dependencies

While libcosmic is written entirely in Rust, some of its dependencies may require shared system library headers to be installed. On Pop!_OS, the following dependencies are all that's necessary to compile a typical COSMIC project:

```sh
sudo apt install cargo cmake just libexpat1-dev libfontconfig-dev libfreetype-dev libxkbcommon-dev pkgconf
```

## Examples

Some examples are included in the [examples](./examples) directory to to kickstart your
COSMIC adventure. To run them, you need to clone the repository with the following commands:

```sh
git clone --recurse-submodules https://github.com/pop-os/libcosmic
cd libcosmic
```

If you have already cloned the repository, run these to sync with the latest updates:

```sh
git fetch origin
git checkout master
git reset --hard origin/master
```

The examples may then be run by their cargo project names, such as `just run application`.

## Cargo Features

Available cargo features to choose from:

- `a11y`: Experimental accessibility support.
- `animated-image`: Enables animated images from the image crate.
- `debug`: Enables addtional debugging features.
- `smol`: Uses smol as the preferred async runtime.
    - Conflicts with `tokio`
- `tokio`: Uses tokio as the preferred async runtime.
    - If unset, the default executor defined by iced will be used.
    - Conflicts with `smol`
- `wayland`: Wayland-compatible client windows.
    - Conflicts with `winit`
- `winit`: Cross-platform and X11 client window support
    - Conflicts with `wayland`
- `wgpu`: GPU accelerated rendering with WGPU.
    - By default, softbuffer is used for software rendering.
- `xdg-portal`: Enables XDG portal dialog integrations.

### Project Showcase

- [COSMIC App Library](https://github.com/pop-os/cosmic-applibrary)
- [COSMIC Applets](https://github.com/pop-os/cosmic-applets)
- [COSMIC Launcher](https://github.com/pop-os/cosmic-launcher)
- [COSMIC Notifications](https://github.com/pop-os/cosmic-notifications)
- [COSMIC Panel](https://github.com/pop-os/cosmic-panel)
- [COSMIC Text Editor](https://github.com/pop-os/cosmic-text-editor)
- [COSMIC Settings](https://github.com/pop-os/cosmic-settings)

## Licence

Licensed under the [Mozilla Public License 2.0](https://choosealicense.com/licenses/mpl-2.0).

## Contact

- [Mattermost](https://chat.pop-os.org/)
- [Lemmy](https://lemmy.world/c/pop_os)
- [Mastodon](https://fosstodon.org/@pop_os_official)
- [Reddit](https://www.reddit.com/r/pop_os/)
- [Twitter](https://twitter.com/pop_os_official)
- [Instagram](https://www.instagram.com/pop_os_official)
