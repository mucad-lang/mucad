# µcad viewer

[![Crates.io](https://img.shields.io/crates/v/microcad-viewer.svg)](https://crates.io/crates/microcad-viewer)
[![Documentation](https://docs.rs/microcad-viewer/badge.svg)](https://docs.rs/microcad-viewer/)

This crate provides the viewer application to view µcad files.

## Requirements

On Debian systems using wayland, install the wayland libraries first before compiling:

```mu
sudo apt install libwayland-dev
```

## Install via cargo

```sh
cargo install microcad-viewer
```

## Run

After install, you can run the microcad viewer by typing `microcad-viewer` into your console.

```sh
microcad-viewer some_file.mu
```

### Run in workspace root using Cargo

```sh
RUST_LOG=info cargo run --bin microcad-viewer --package microcad-viewer -- examples/csg_cube
```

## ❤️ Support the project

This crate is part of the [microcad project](https://microcad.xyz).

If you like this project, you can help us spend more time on it by donating:

<a href="https://opencollective.com/microcad/donate" target="_blank">
<img src="https://opencollective.com/microcad/donate/button@2x.png?color=blue" width=300 />
</a>
