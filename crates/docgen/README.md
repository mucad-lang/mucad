# µcad docgen

This crate implements documentation generation for built-in libraries and µcad libraries.
Currently, only the generation of mdbook documentation is supported.

To check out the API and test the built-in documentation generation, run the following command:

```sh
cargo run --bin microcad-docgen-builtin --package microcad-docgen --features="bin_builtin"  -- target/builtin
```

 ## ❤️ Support the project

This crate is part of the [microcad project](https://microcad.xyz).

If you like this project, you can help us spend more time on it by donating:

<a href="https://opencollective.com/microcad/donate" target="_blank">
<img src="https://opencollective.com/microcad/donate/button@2x.png?color=blue" width=300 />
</a>
