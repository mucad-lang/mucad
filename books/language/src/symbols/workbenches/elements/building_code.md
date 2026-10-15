# Building Code

The *building code* is executed after any initialization.
Usually it produces one or many 2D or 3D objects on base of the given
*building plan*.

[![test](.test/code.svg)](.test/code.log)

```mu,code
sketch Wheel(radius: Length, thickness = 5mm) {
    // building code starts here
    use std::geo2d::Circle;
    Circle(radius = radius + thickness) - Circle(radius)
}
Wheel(radius = 1cm);
```

If *initializers* were defined the *building code* starts below them.

[![test](.test/code_post_init.svg)](.test/code_post_init.log)

```mu,code_post_init
sketch Wheel(radius: Length, thickness = 5mm) {
    // initializer code
    use std::geo2d::Circle;
    // initializer
    init(diameter: Length) {
        radius = diameter / 2;
    }

    // building code starts here
    std::geo2d::Circle(radius);
}
Wheel(radius = 1cm);
```

## Restrictions

### Illegal statements within workbenches

It's **not allowed** to use the `sketch`, `part`, `op`, `return` nor `mod` statements within workbench code:

[![test](.test/illegal_workbench_statement_sketch.svg)](.test/illegal_workbench_statement_sketch.log)

```mu,illegal_workbench_statement_sketch?result=fail
sketch Wheel(radius: Length) {
    sketch A() {} // error
    part B() {} // error
    op C() {} // error
}
```

[![test](.test/illegal_workbench_statement_mod.svg)](.test/illegal_workbench_statement_mod.log)

```mu,illegal_workbench_statement_mod?result=fail
sketch Wheel(radius: Length) {
    mod m {} // error
}
```

[![test](.test/illegal_workbench_statement_return.svg)](.test/illegal_workbench_statement_return.log)

```mu,illegal_workbench_statement_return?result=fail
sketch Wheel(radius: Length) {
    return; // error
}
```
