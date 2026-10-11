# Documentation

## Code Comments (`//`, `/* ... */`)

By using `//` or `/* ... */` you may insert comments within the code.

[![test](.test/comment.svg)](.test/comment.log)

```mu,comment
// This is a line comment...

/* This is a block comment */

/* Block comments my have...
   ...multiple lines. */

std::print("Hello, "); // Line comments can be appended to a line
std::print( /* Block comments can be placed almost anywhere */ "world!");
```

## Doc Comments (`///`, `//!`)

You may also use comments to attribute your code with documentation.
There are outer (`///`) and inner `//!` doc comments.

### Outer doc comments (`///`)

By placing a comment with `///` above a symbol definition you can attribute
your code with documentation.
Markdown may be used to shape sections or format text.

[![test](.test/outer_doc_comment.svg)](.test/outer_doc_comment.log)

```mu,outer_doc_comment
/// A function which returns what it gets.
///
/// It simply returns the **same** value...
///
/// ...as it got from the *parameter*.
///
/// ## Arguments
///
/// - `n`: input value
///
/// ## Returns
///
/// Output value.
fn f(n: Integer) -> Integer { n }

// usual comment for non-symbols
f(1);
```

Till the first empty line text will be interpreted as a summary for documentation,
all other lines will build the detailed description.

The above function `f` will be documented with the following markdown output:

> A function which returns what it gets.
>
> It simply returns the **same** value...
>
> ...as it got from the *parameter*.
>
> ## Arguments
>
> - `n`: input value
>
> ## Returns
>
> Output value.

### Inner doc comments (`//!`)

Inner doc comments are used to document code inside a source file:

[![test](.test/inner_doc_comment.svg)](.test/inner_doc_comment.log)

```mu,inner_doc_comment
//! This inner doc comment documents the whole source file.

fn f(n: Integer) -> Integer { n }

f(1);
```
