# Introduction

[libcosmic][toolkit] is the platform toolkit for [COSMIC](cosmic)—a GUI toolkit which empowers everyone to build COSMIC-themed applets and applications with ease. Based on the cross-platform [iced][iced] GUI library—which it utilizes for its runtime and rendering primitives—the COSMIC toolkit features personalizable desktop theming, a responsive widget library, a configuration system, platform integrations, and its own interface guidelines for building consistent and responsive applications.

As a Rust-based GUI toolkit, experience with [Rust](rust) is required. Rust's rich type system and language features are key to what makes the COSMIC toolkit a much friendlier developer experience—enabling secure, reliable, and efficient applications to be developed at a faster pace than would be possible otherwise. For those interested in learning Rust, there are a lot of good resources available: [Learn Rust in a Month of Lunches][month-of-lunches], [Rust in Action][rust-in-action], [Rust by Example][rust-by-example], the official [Rust Book][rust-book], and [Rustlings][rustlings].

Although the toolkit was created for the COSMIC desktop environment, it is also cross-platform, and thus it can be used to build COSMIC-themed applications for Linux (X11 & Wayland), [Redox OS](redox-os), Windows, and Mac. Even mobile platforms could be a possibility someday. One of the goals of libcosmic is to enable the creation of a cross-platform ecosystem of applications that are easy to port from one OS to another. We would also welcome any that would like to build their own OS experiences with the COSMIC toolkit.

[cosmic]: https://github.com/pop-os/cosmic-epoch
[iced]: https://iced.rs/
[month-of-lunches]: https://www.manning.com/books/learn-rust-in-a-month-of-lunches
[redox-os]: https://redox-os.org/
[rust]: https://www.rust-lang.org/
[rust-book]: https://doc.rust-lang.org/stable/book/
[rust-by-example]: https://doc.rust-lang.org/rust-by-example/
[rust-in-action]: https://www.manning.com/books/rust-in-action
[rustlings]: https://github.com/rust-lang/rustlings
[toolkit]: https://github.com/pop-os/libcosmic
