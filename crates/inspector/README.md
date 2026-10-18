# µcad inspector

[![Crates.io](https://img.shields.io/crates/v/mucad-inspector.svg)](https://crates.io/crates/mucad-inspector)
[![Documentation](https://docs.rs/mucad-inspector/badge.svg)](https://docs.rs/mucad-inspector/)

This crate provides an experimental inspector app to inspect µcad files.

## Install via cargo

You can install this application using cargo.

```sh
cargo install mucad-inspector
```

## Run

After installing, you can run the mucad inspector by typing `mucad-inspector` into your console.

```sh
mucad-inspector some_file.mu
```

### Run workspace root using Cargo

```sh
RUST_LOG=info cargo run --bin mucad-inspector --package mucad-inspector -- examples/csg_cube.mu
```

## ❤️ Support the project

This crate is part of the [mucad project](https://mucad.org).

If you like this project, you can help us spend more time on it by donating:

<a href="https://opencollective.com/mucad/donate" target="_blank">
<img src="https://opencollective.com/mucad/donate/button@2x.png?color=blue" width=300 />
</a>
