# µcad inspector

[![Crates.io](https://img.shields.io/crates/v/microcad-inspector.svg)](https://crates.io/crates/microcad-inspector)
[![Documentation](https://docs.rs/microcad-inspector/badge.svg)](https://docs.rs/microcad-inspector/)

This crate provides an experimental inspector app to inspect µcad files.

## Install via cargo

You can install this application using cargo.

```sh
cargo install microcad-inspector
```

## Run

After installing, you can run the microcad inspector by typing `microcad-inspector` into your console.

```sh
microcad-inspector some_file.mu
```

### Run workspace root using Cargo

```sh
RUST_LOG=info cargo run --bin microcad-inspector --package microcad-inspector -- examples/csg_cube.mu
```

## ❤️ Support the project

This crate is part of the [microcad project](https://microcad.xyz).

If you like this project, you can help us spend more time on it by donating:

<a href="https://opencollective.com/microcad/donate" target="_blank">
<img src="https://opencollective.com/microcad/donate/button@2x.png?color=blue" width=300 />
</a>
