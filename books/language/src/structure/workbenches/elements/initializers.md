# Initializers

*Initializers* are a way to define alternative parameters to create the
[*building plan*](building_plan.md).
An Initializer is defined with the keyword `init` and a following *parameter list*.
One may define multiple initializers which must have different parameter lists.

However, if an initializer is used, all properties from the building plan must
be initialized (except those with *default values*).

[![test](.test/init_property.svg)](.test/init_property.log)

```mu,init_property
sketch Wheel(radius: Length, thickness = 5mm) {
    use std::geo2d::Circle;

    // initializer with diameter
    init(diameter: Length, thickness = 5mm) {
        // must set property `radius` from building plan
        radius = diameter / 2;

        // `thickness` (from the building plan) does not need
        // to be set, because it was automatically set by
        // parameter of this initializer
    }

    // Now radius and thickness can be used
    Circle(radius = radius + thickness) - Circle(radius);
}
// call with building plan
Wheel(radius = 1.5cm, thickness = 2mm);
// call with initializer and use default thickness
Wheel(diameter = 1.5cm);
```

Output
  :![output](.test/init_property-out.svg)

## Restrictions

### Building plan must be initialized

If the *building plan* is not fully initialized by an initializer
you will get an error:

[![test](.test/missed_property.svg)](.test/missed_property.log)

```mu,missed_property#fail
sketch Wheel(radius: Length, thickness = 5mm) {
    use std::geo2d::Circle;

    init(thickness: Length) {} // error: misses to set radius from building plan

    Circle(radius = radius + thickness) - Circle(radius); // error: radius is missing
}
Wheel(thickness = 1cm);
```

### Building plan properties with default values

Parameters of a workbench's building plan which have a *default value* do not
need to be set in the initializers.

[![test](.test/building_plan_defaults.svg)](.test/building_plan_defaults.log)

```mu,building_plan_defaults#todo
sketch Wheel(radius: Length, thickness = 5mm) {
    use std::geo2d::Circle;

    init(diameter: Length) {
        radius = diameter / 2;
        // thickness has been set automatically by the default in the building plan
    }

    Circle(radius = radius + thickness) - Circle(radius);
}

Wheel(diameter = 1cm);
```

Output
  :![output](.test/building_plan_defaults-out.svg)

### Building plan cannot be accessed within initializers

You cannot read building plan items from within initializers.

[![test](.test/no_building_plan_in_initializers.svg)](.test/no_building_plan_in_initializers.log)

```mu,no_building_plan_in_initializers#todo
sketch Wheel(radius: Length, thickness = 5mm) {
    use std::geo2d::Circle;

    init(diameter: Length, thickness = 5mm) {
        _ = radius; // error: cannot read radius here
        radius = diameter / 2; // instead you need to set it
    }

    Circle(radius = radius + thickness) - Circle(radius);
}
Wheel(diameter = 1cm);
```

### Initializers with parameters from building plan

If you use parameter names in an initializer which already are used in the
building plan, they will automatically become properties and cannot be set
second time.

[![test](.test/no_building_plan_same_name.svg)](.test/no_building_plan_same_name.log)

```mu,no_building_plan_same_name#fail
sketch Wheel(radius: Length, thickness: Length) {
    use std::geo2d::Circle;

    init(radius: Length) {
        // radius property has already been set by building plan
        radius = radius * 2; // error: it cannot be set a second time
        thickness = 5mm;
    }
    Circle(radius = radius + thickness) - Circle(radius);
}
// Use initializer
Wheel(radius = 1cm);
```

Types must match when using a name from building plan in initializer parameters.

[![test](.test/no_building_plan_same_name_different_type.svg)](.test/no_building_plan_same_name_different_type.log)

```mu,no_building_plan_same_name_different_type#todo
sketch Wheel(radius: Length, thickness: Length) {
    use std::geo2d::Circle;

    init(radius: Scalar, outer: Length) {
        // error: radius is already a Length in building plan
        thickness = outer - (radius * 1mm);
    }

    Circle(radius = radius + thickness) - Circle(radius);
}
// Use initializer
Wheel(radius = 1.0, outer = 1cm);
```

### No code between initializers

It's not allowed to write any code between *initializers*.

[![test](.test/code_between_initializers.svg)](.test/code_between_initializers.log)

```mu,code_between_initializers#fail
sketch Wheel(radius: Length, thickness = 5mm) {
    use std::geo2d::Circle;

    init(width: Length, thickness = 5mm) {
        radius = width / 2;
    }
    radius = 1; // error: code between initializers not allowed
    init(height: Length, thickness = 5mm) {
        radius = height / 2;
    }

    Circle(radius = radius + thickness) - Circle(radius);
}
Wheel(radius = 1cm);
```
