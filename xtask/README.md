# µcad xtask utilities

## Run doc tests

```sh
cargo xtask doctest books/language > results.md
```

## Check README commands

Run the shell commands of all `README.md` files one by one, to check them manually:

```sh
cargo xtask check-readme
```

Each command can be run, skipped or the check quit. Commands run from the repository root,
a `cd` changes the directory for the following commands of its code block.
After a command ran, you decide if it worked. A summary lists all commands with their results.

Only list the commands, e.g. of one directory:

```sh
cargo xtask check-readme editors --list
```

## ❤️ Support the project

This crate is part of the [mucad project](https://mucad.org).

If you like this project, you can help us spend more time on it by donating:

<a href="https://opencollective.com/mucad/donate" target="_blank">
<img src="https://opencollective.com/mucad/donate/button@2x.png?color=blue" width=300 />
</a>
