# List type

> [!NOTE]
> Arrays are quite involved in the [multiplicity concept](../../flow/calls/multiplicity.md)
> so you might want to read about this too.

## Declaration

An list is an ordered collection of values with a common type.

[![test](.test/arrays.svg)](.test/arrays.log)

```mu,list_examples
a: [Integer] = [1, 2, 3, 4, 5];
b: [Scalar] = [1.42, 2.3, 3.1, 4.42, 5.23];
c: [Length] = [1.42mm, 2.3m, 3.1cm, 4.42cm, 5.23m];
d: [String] = ["one", "two", "three", "four", "five"];
e: [Model] = [
    Circle(radius = 1cm),
    std::geo3d::Sphere(radius = 1cm),
];
```

Of course these type declarations can be skipped (e.g. `a = [ 1, 2, 3, 4, 5 ]`).

### Unit bundling

Lists support *unit bundling*, which means the you can write a unit behind the brackets.

[![test](.test/array_unit_bundling.svg)](.test/array_unit_bundling.log)

```mu,list_unit_bundling
assert_eq([1mm, 2mm, 3mm], [1, 2, 3]mm);
```
