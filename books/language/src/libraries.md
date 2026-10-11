# Included Libraries

With `microcad` comes with a *builtin* and a *standard* library.

## µcad Builtin Library

The *µcad builtin library* is written in Rust (and still a little C) and brings
mathematical calculation functions, model processing and rendering capabilities
into µcad.

The µcad builtin library is available in the global module `__builtin` and is self
documented in a [reference book](https://docs.microcad.xyz/builtin/book).

[![test](.test/lib_builtin.svg)](.test/lib_builtin.log)

```mu,lib_builtin
use __builtin::print;

print("Hello, µcad builtin library!");
```

## µcad Standard Library

The *µcad standard library* is written in µcad and provides a convenient
interface to the µcad builtin library.

The µcad standard library is available in the global module `std` and is self
documented in a [reference book](https://docs.microcad.xyz/std/book).

[![test](.test/lib_std.svg)](.test/lib_std.log)

```mu,lib_std
use std::print;

print("Hello, µcad standard library!");
```
