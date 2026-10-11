# Quantities

The term *quantities* bundles a set of [*quantity types*](types.md) which are
basically *floating point[^float] values* with physical *units* (e.g.  Meter, Liters,...) attached.

[^float]: see [wikipedia about floating point](https://en.wikipedia.org/wiki/Floating-point_arithmetic)

[![test](.test/quantity_types_number_literals.svg)](.test/quantity_types_number_literals.log)

```mu,quantity_types_number_literals
// declare variable `height` of type `Length` to 1.4 Meters
height = 1.4m;

// use as *default* value in parameter list
fn f(height = 1m) {}

// calculate a `Length` called `width` by multiplying the
// `height` with `Scalar` `2` and add ten centimeters
width = height * 2 + 10cm;
```
