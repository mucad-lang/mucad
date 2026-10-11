# Model Assignments

Model assignments look like [value assignments](value.md) but instead having a
value on the right side they store a model.

[![test](.test/model_assignment.svg)](.test/model_assignment.log)

```mu,model_assignment
m = std::geo2d::Circle(radius = 10mm); // assign the model of a circle into m
std::debug::assert_eq([m.radius, 10mm]); // access property radius of m
```

Using a model as a value or vice versa does not work without further operations.

[![test](.test/model_assignment_cross.svg)](.test/model_assignment_cross.log)

```mu,model_assignment_cross#fail
m = std::geo2d::Circle(radius = 10mm); // assign the model of a circle into m
std::geo2d::Circle(radius = m); // error: cannot use m as value
```

## Restrictions

### No Shadowing

[Like with value assignments](value.md#no-shadowing) so-called "shadowing"
(reusing a *name*) is prohibited.
Another assignment of a variable with the same name without an additional scope is
prohibited.

### Not in modules

Model assignments are not available in modules:

[![test](.test/assignment_model_module.svg)](.test/assignment_model_module.log)

```mu,assignment_model_module#fail
mod my_module {
    a = std::geo2d::Circle(radius = 1mm); // error
}
```

### Not in initialization code

Model assignments are not available in workbenches' initialization code:

[![test](.test/assignment_model_workbench.svg)](.test/assignment_model_workbench.log)

```mu,assignment_model_workbench#fail
sketch MySketch() {
    a = std::geo2d::Circle(radius = 1mm); // error
    init(_x: Scalar) {}
}

MySketch();
```
