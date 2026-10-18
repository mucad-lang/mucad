# mucad

[![Crates.io](https://img.shields.io/crates/v/mucad.svg)](https://crates.io/crates/mucad)
[![Documentation](https://docs.rs/mucad/badge.svg)](https://docs.rs/mucad/)

This crate provides a command line interface for µcad language.

**Note**: This project is in an early stage of development and is not yet feature complete!

## Quick Start

To process a µcad file and export its geometry into an STL, use the following command:

```sh
mucad export ./examples/bricks/brick
```

This will export the 3D geometry description in `./examples/bricks/brick.µcad` as a `brick.stl` file.

## Installation

### Install µcad via cargo

To install the latest release of *µcad* via *cargo*, type:

```sh
cargo install mucad
```

## Run

Start the mucad CLI by typing `mucad` into your console.

```sh
mucad
```

You're now ready to use µcad!

## Command line usage

To generate an STL model file, use the `export` command (with an optional output file name):

```sh
mucad export ./examples/bricks/brick
```

This will export the geometry into a `brick.stl` file.

## Generate documentation

You can generate documentation with `mucad doc` for source files.

### Generate mdbook for built-in library

```sh
mucad doc -g mdbook __builtin --output books/builtin
```

### Generate mdbook for µcad standard library

```sh
mucad doc -g mdbook 'crates/std/lib/std/mod.µcad' --output books/std`
```

## Resources

* Documentation: <https://docs.mucad.org/latest/language/>
* µcad Viewer: <https://crates.io/crates/mucad-viewer/>
*

## ❤️ Support the project

This crate is part of the [mucad project](https://mucad.org).

If you like this project, you can help us spend more time on it by donating:

<a href="https://opencollective.com/mucad/donate" target="_blank">
<img src="https://opencollective.com/mucad/donate/button@2x.png?color=blue" width=300 />
</a>
