# Parts

Parts are *workbenches* that are used to create 3D *models*.
They are named in `PascalCase`:

[![test](.test/part_basic.svg)](.test/part_basic.log)

```mu,part_basic
part MyPart(radius: Length) {
    use std::geo3d::*;
    Sphere(radius) - Cube(radius)
}

MyPart(1cm);
```

Like all workbenches parts can have several [workbench elements](../elements/).

## Restrictions

### Parts cannot generate 2D models

You will get an error if you generate a 2D model with a part:

[![test](.test/part_2d.svg)](.test/part_2d.log)

```mu,part_2d?result=fail
part MyPart(radius: Length) {
    std::geo2d::Circle(radius); // error: Circle is not a 3D primitive
}

MyPart(1cm);
```
