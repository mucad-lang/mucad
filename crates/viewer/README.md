# µcad viewer

[![Crates.io](https://img.shields.io/crates/v/mucad-viewer.svg)](https://crates.io/crates/mucad-viewer)
[![Documentation](https://docs.rs/mucad-viewer/badge.svg)](https://docs.rs/mucad-viewer/)

This crate provides the viewer application to view µcad files.

## Requirements

On Debian systems using wayland, install the wayland libraries first before compiling:

```mu
sudo apt install libwayland-dev
```

## Install via cargo

```sh
cargo install mucad-viewer
```

## Run

After install, you can run the mucad viewer by typing `mucad-viewer` into your console.

```sh
mucad-viewer some_file.mu
```

### Run in workspace root using Cargo

```sh
RUST_LOG=info cargo run --bin mucad-viewer --package mucad-viewer -- examples/csg_cube
```

## ❤️ Support the project

This crate is part of the [mucad project](https://mucad.org).

If you like this project, you can help us spend more time on it by donating:

<a href="https://opencollective.com/mucad/donate" target="_blank">
<img src="https://opencollective.com/mucad/donate/button@2x.png?color=blue" width=300 />
</a>
