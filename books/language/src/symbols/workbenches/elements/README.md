# Workbench Elements

A *workbench* in general consists of the following elements:

- A leading keyword: `part`, `sketch`, or `op`,
- an **identifier** that names the workbench,
- a [**building plan**](building_plan.md) defined by a *parameter list* following the identifier,
- optional [**initialization code**](init_code.md), which is placed and executed before any *initializer*,
- optional [**initializers**](initializers.md), offering alternative ways to initialize the *building plan*,
- optional [**functions**](../functions/workbench_functions.md), acting as local subroutines with their own parameters and code bodies,
- optional [**properties**](properties.md), accessible from outside and set from the inside,
- and typically some [**building code**](building_code.md), which runs after all initialization steps and generates the final *objects*.

The following code demonstrates (on the basis of a `sketch`) most of these elements which we will discuss in the following pages in detail.

[![test](.test/part_declaration.svg)](.test/part_declaration.log)

```mu,part_declaration
// Sketch with a `radius` as building plan.
// Which will automatically become a property.
sketch Wheel(radius: Length) {
    const FACTOR = 2;

    // Initializer #1
    init(diameter: Length) {
        // must set `radius`
        radius = diameter / FACTOR;
    }

    // No code in between!

    // Initializer #2
    init(r: Length) {
        // must set `radius`
        radius = r;
    }

    // Function (sub routine)
    fn into_diameter(r: Length) {
        return r * FACTOR;
    }

    // Building code...

    // Set a property which can be seen from outside.
    prop diameter = into_diameter(radius);

    // Hidden property `r`
    r = radius;

    // Create a circle.
    Circle(r);
}

// Call sketch with diameter.
d = Wheel(diameter = 2cm);
// Check radius property.
assert_eq(d.radius, 1cm);

// Call sketch with radius.
r = Wheel(radius = 2cm);
// Check diameter property.
assert_eq(r.diameter, 4cm);

r - d;
```

Output
  :![output](.test/part_declaration-out.svg)
