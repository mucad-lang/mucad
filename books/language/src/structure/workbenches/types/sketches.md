# Sketches

Sketches are similar to *parts* but in two dimensions only.
They may be extruded into three-dimensional [parts](parts.md) by using
[operations](operations.md).
Sketches are named in `PascalCase`:

[![test](.test/sketch_basic.svg)](.test/sketch_basic.log)

```mu,sketch_basic
sketch MySketch(radius: Length) {
    use std::geo2d::*;
    Circle(radius) - Rect(size = radius);
}

MySketch(1cm);
```

The output is a 2D sketch:

![test](.test/sketch_basic-out.svg)

## Restrictions

### Sketches cannot generate 3D models

You will get an error if you generate a 3D model with a sketch:

[![test](.test/sketch_3d.svg)](.test/sketch_3d.log)

```mu,sketch_3d#fail
sketch MySketch(radius: Length) {
    use std::geo3d::*;
    Sphere(radius) - Cube(size = radius); // error: Sphere and Cube are 3D
}

MySketch(1cm);
```
