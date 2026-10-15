# Properties

There are two ways to declare *Properties*:

- as parameter of the building plan or
- in an assignment within the build code by using the keyword `prop`.

In the following example we declare a building plan which consists of a `radius` which will automatically be a property:

[![test](.test/property.svg)](.test/property.log)

```mu,property
// `outer` will automatically become a property because
// it is declared in the building plan:
sketch Wheel(outer: Length) {
    use std::geo2d::Circle;

    // `inner` is declared as property
    prop inner = outer / 2;

    // generate wheel (and use property `outer` and `inner`)
    Circle(radius = outer) - Circle(radius = inner);
}

// evaluate wheel
t = Wheel(1cm);

// extract and display `outer` and `inner` from generated wheel
std::print("outer: {t.outer}");
std::print("inner: {t.inner}");
```

Output
  : ```
    outer: 10mm
    inner: 5mm
    ```

If you remove the `prop` keyword you will fail at accessing `inner`:

[![test](.test/property_wrong.svg)](.test/property_wrong.log)

```mu,property_wrong?result=fail
sketch Wheel(outer: Length) {
    use std::geo2d::Circle;

    // `inner` is declared as variable and may not be read
    // from outside this workbench
    inner = outer / 2;

    Circle(radius = outer) - Circle(radius = inner);
}

t = Wheel(outer = 1cm);

// you can still extract and display `outer`
std::print("outer: {t.outer}");
// but you cannot access `inner` anymore
std::print("inner: {t.inner}"); // error
```

## Restrictions

### No prop within initializer

You may not define a property within an initializer.

[![test](.test/property_no_prop_in_initializer.svg)](.test/property_no_prop_in_initializer.log)

```mu,property_no_prop_in_initializer?result=fail
sketch Wheel(radius: Length, thickness = 5cm) {
    init(radius: Length, inner: Length) {
        thickness = radius - inner;

        prop center = radius - inner; // error: do not use prop here
    }

    prop center = radius - thickness / 2; // here it's ok
}
center = Wheel(radius = 1cm, inner = 0.5cm).center;

std::assert_eq(center, 1.75cm);
```

Output
  :![test](.test/property_no_prop_in_initializer-out.svg)

### No prop within initialization code

Also you may not use `prop` within initialization code.

[![test](.test/property_no_prop_in_init_code.svg)](.test/property_no_prop_in_init_code.log)

```mu,property_no_prop_in_init_code?result=fail
sketch Wheel(outer: Length) {
    prop max = 100; // error: do not use prop here

    init(inner: Length) {
        outer = inner * 2;
    }
}
Wheel(inner = 0.5cm);
```
