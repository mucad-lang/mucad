# Property Assignments

Property assignments define additional[^additional]properties of a workbench.
The may appear anywhere within the building code of a workbench and can then be read from outside.

[^additional]: Additional to properties which are automatically generated from a workbench's building plan.

[![test](.test/prop_assignment.svg)](.test/prop_assignment.log)

```mu,prop_assignment
sketch MySketch(radius: Length) {
    prop diameter = radius * 2;
    std::geo2d::Circle(radius);
}
std::debug::assert_eq([MySketch(5cm).diameter, 10cm])
```

## Restrictions

### Not in source files

[![test](.test/prop_assignment_source.svg)](.test/prop_assignment_source.log)

```mu,prop_assignment_source#fail
prop diameter = radius * 2; // error: not in source file
```

### Not in functions

[![test](.test/prop_assignment_fn.svg)](.test/prop_assignment_fn.log)

```mu,prop_assignment_fn#fail
fn f() {
    prop diameter = radius * 2; // error: not in functions
}

f();
```

### Not in initialization code

[![test](.test/prop_assignment_init.svg)](.test/prop_assignment_init.log)

```mu,prop_assignment_init#fail
sketch MySketch(radius: Length) {
    prop diameter = radius * 2; // error: not in initialization code

    init() {
        radius = 1;
    }
    std::geo2d::Circle(radius);
}
std::debug::assert_eq([MySketch(5cm).diameter, 10cm])
```

### Not in initializers

[![test](.test/prop_assignment_initializer.svg)](.test/prop_assignment_initializer.log)

```mu,prop_assignment_initializer#fail
sketch MySketch(radius: Length) {
    init() {
        radius = 1;
        prop diameter = radius * 2; // error: not in initializer
    }
    std::geo2d::Circle(radius);
}
MySketch(5cm)
```
