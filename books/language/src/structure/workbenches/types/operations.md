# Operations

> [!NOTE]
> Do not confuse *operations* with [*operators*](../../../expressions/operators.md)

*Operations* process 2D or 3D geometries into 2D or 3D geometries.
Unlike *sketches* or *parts* they are named in `snake_case`.

Actual operations are workbenches that process *input models* into *output models*.
So the following `nop` operation would be a neutral operation which just
passes-through the original input model:

[![test](.test/op_example.svg)](.test/op_example.log)

```mu,op_example
// define operation nop without parameters
op nop() { @input }

// use operation `nop` on a circle results in the same circle
std::geo2d::Circle(radius = 1cm).nop();
```

Output
  :![test](.test/op_example-out.svg)

## `@input`

`@input` is a placeholder to tell where the input nodes of the operation shall
be inserted.

An operation can have multiple children when they are bunched together in a
group.
In the following example `punshed_disk` awaits a group of exactly two children.

[![test](.test/input.svg)](.test/input.log)

```mu,input#todo
// define operation which takes multiple items
op punched_disk() {
    // check number of input models
    if @input.count() == 2 {
        // make hole by subtracting both inputs
        @input.subtract();
    } else {
        std::error("punched_disk must get exactly two objects");
    }
}

// use operation punch_disk with two circles
{
    std::geo2d::Circle(radius = 1cm);
    std::geo2d::Circle(radius = 2cm);
}.punched_disk();
```

Output
  :![test](.test/input-out.svg)

Like other workbenches operations can have parameters too:

[![test](.test/parameters.svg)](.test/parameters.log)

```mu,parameters#todo
// define operation which takes multiple items
op punch_disk(radius: Length) {
    if @input.count() == 1 {
        {
            @input;
            std::geo2d::Circle(radius);
        }.subtract();
    } else {
        std::error("punch_disk must get one object");
    }
}

// use operation punch_disk on a circle
{
    std::geo2d::Circle(radius = 2cm);
}.punch_disk(radius = 1cm);
```

Output
  :![test](.test/parameters-out.svg)
