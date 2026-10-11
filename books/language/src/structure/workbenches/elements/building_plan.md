# Building Plan

The *building plan* is defined by a *parameter list* that follows the workbench's
*identifier*.
All *parameters* in that list will automatically become [properties](properties.md) of the
workbench when it is invoked.
These properties can be accessed within the [*building code*](building_code.md), inside functions,
or externally.

The following code demonstrates this using a `sketch` with a single parameter
in the building plan, called `radius`, of type `Length`:

[![test](.test/building_plan.svg)](.test/building_plan.log)

```mu,building_plan
// sketch with a radius as building plan
sketch Wheel(radius: Length, thickness = 5mm) {
    use std::geo2d::Circle;

    // access property radius from the building plan
    Circle(radius = radius + thickness) - Circle(radius)
}

// access property radius of a Wheel
w = Wheel(1cm);
// render Wheel
w;

// check if r is 5cm an thickness equals the default (1cm)
std::debug::assert_eq([w.radius, 1cm]);
std::debug::assert_eq([w.thickness, 5mm]);
```

Output
  :![output](.test/building_plan-out.svg)
