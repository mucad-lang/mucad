# Workbench Functions

A [workbench](../workbenches/) can contain functions.
These functions can be used within the workbench, but **not** from outside it.

Here is an example which generates a punched disk of a given radius using a
function `inner()`:

[![test](.test/workbench_example.svg)](.test/workbench_example.log)

```mu,workbench_example
sketch PunchedDisk(radius: Length) {
    use std::geo2d::Circle;

    // function to calculate inner from radius
    fn inner(radius: Length) { radius / 2 }

    // generate donut (and call inner)
    Circle(radius) - Circle(radius = inner(radius));
}

PunchedDisk(1cm);
```

## Restrictions

There are some restrictions for *workbench functions*:

### No public workbench functions

Trying to make them public with the keyword `pub` will result into an error:

[![test](.test/workbench_pub.svg)](.test/workbench_pub.log)

```mu,workbench_pub?result=fail
sketch PunchedDisk(radius: Length) {
    use std::geo2d::Circle;

    pub fn inner() { radius / 2 } // error: cant use pub fn inside workbench

    Circle(radius) - Circle(radius = inner());
}

PunchedDisk(1cm);
```

### No `prop` in workbench functions nor initializers

You cannot create *workbench properties* in *function bodies*.

[![test](.test/workbench_fn_prop.svg)](.test/workbench_fn_prop.log)

```mu,workbench_fn_prop?result=fail
sketch PunchedDisk(radius: Length) {
    use std::geo2d::Circle;

    fn inner() {
        prop hole = radius / 2; // error: prop not allowed in function
        hole
    }

    prop hole = radius / 2; // correct prop definition

    Circle(radius) - Circle(radius = inner());
}

PunchedDisk(1cm);
```

Also the `prop` keyword is not allowed in *initializers*.
Instead, the properties of the building plan must be set directly, without
using the `prop` keyword.

[![test](.test/workbench_init_prop.svg)](.test/workbench_init_prop.log)

```mu,workbench_init_prop?result=fail
sketch PunchedDisk(radius: Length) {
    use std::geo2d::Circle;

    init(diameter: Length) {
        prop radius = diameter / 2; // error: prop not allowed in init
    }

    init(d: Length) {
        radius = d / 2; // correct way to set radius
    }

    // Accessing property in a function is ok
    fn inner() { radius / 2 }

    Circle(radius) - Circle(radius = inner());
}

PunchedDisk(diameter = 1cm);
PunchedDisk(d = 1cm);
```
