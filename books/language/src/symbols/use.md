# Use Statements

The use statement imports symbols from one module into another’s local namespace.
This allows symbols to be referenced by their short names rather than their fully qualified names, reducing boilerplate and improving code readability.

The following example which uses two *parts* of `std::geo3d` shows the problem with long names:

[![test](.test/none.svg)](.test/none.log)

```mu,none
std::geo3d::Sphere(radius = 40mm);
std::geo3d::Cube(size = 40mm);
```

There are several ways to solve this with `use`.

## `use` Statement

A use statement creates a local alias for a specific symbol. While the initial declaration takes space, it significantly shortens subsequent calls.

Looking at the example below, using `use` does not seem any shorter, but if we would use `Sphere` and `Cube` repeatedly,
this would shorten things quite a lot:

[![test](.test/use.svg)](.test/use.log)

```mu,use
use std::geo3d::Sphere;
use std::geo3d::Cube;

Sphere(radius = 4mm);
Cube(size = 40mm);
```

Alternatively, you may also use an entire **module** `geo3d` and to get rid of the `std::` prefix part within the names:

[![test](.test/use_module.svg)](.test/use_module.log)

```mu,use_module
use std::geo3d;

geo3d::Sphere(radius = 40mm); // Drops the `std::` prefix.
```

## `use ... as` Statement

Internally, every *use statement* defines an *alias* with an *identifier*
(e.g. `Sphere`) and a *target symbol* it points to (e.g. `std::geo3d::Sphere`).

To prevent name collisions or to improve clarity, you can provide a custom *identifier* using the `as` keyword:

[![test](.test/use_as.svg)](.test/use_as.log)

```mu,use_as
use std::geo3d::Sphere as Ball;

Ball(radius = 4mm);
```

Of course you can use `use as` with a whole *module*:

[![test](.test/use_as_module.svg)](.test/use_as_module.log)

```mu,use_as_module
use std::geo3d as space;

space::Sphere(radius = 4mm);
```

## `use *` Statement

The shortest way to use many symbols from one module is to use `*` as *target*.
The following example defines aliases for **all** symbols of `std::geo3d`.

[![test](.test/use_all.svg)](.test/use_all.log)

```mu,use_all
use std::geo3d::*;

Sphere(radius = 4mm);
Cube(size = 40mm);
Torus(major_radius = 40mm, minor_radius = 20mm);
```

The disadvantage of using `*` is that it increases the risk of name conflicts
between your code and the aliased symbols, some of which you might not even use.

## Public Use Statement

The `pub use` statement does not only make the *target symbol* visible within
the module in which it is defined, but from the outside, too.
See the [visibility section](../visibility.md) for information.

[![test](.test/use_statement_pub.svg)](.test/use_statement_pub.log)

```mu,use_statement_pub
mod my {
    pub use std::geo3d::Sphere;
    pub use std::geo3d::Cube;
}

my::Sphere(radius = 4mm);
my::Cube(size = 40mm);
```
