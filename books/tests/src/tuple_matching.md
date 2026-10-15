# Tuple Argument Matching

The concept behind named tuple argument matching is to allow functions to accept
subsets of parameters in a bundle.
This makes it easy to pre-configure parts of arguments:

[![test](.test/tuple_match.svg)](.test/tuple_match.log)

```mu,tuple_match?mode=todo
// Function with three parameters: x, y, and z
fn f( x: Length, y: Length, z: Length ) {}

// Since we do not want to change x and y in the following statements,
// we prepare a tuple named plane:
plane = (x=1cm, y=2cm);

// Then we pass plane to f() three times with different z values
f( plane, z=3cm);
f( plane, z=6cm);
f( plane, z=9cm);
```

The same function can be called in various ways using named tuples:

[![test](.test/tuple_match_variants.svg)](.test/tuple_match_variants.log)

```mu,tuple_match_variants?mode=todo
fn f( x: Length, y: Length, z: Length ) {}

// Every parameter given by name
f( x=1cm, y=2cm, z=3cm);

// Parameters given by named tuple
f( (x=1cm, y=2cm, z=3cm) );

// Parameters given by named tuple variable
p = (x=1cm, y=2cm, z=3cm);
f( p );

// Parameters given by named tuple and a single value
f( (x=1cm, y=2cm), z=3cm );
f( y=2cm, (x=1cm, z=3cm) );

// Parameters given by named tuple variable and a single value
q = (x=1cm, y=2cm);
f( q, z=3cm );
```

## Tuple Multiplicity

Another example uses an array of tuples and produces the same output:

[![test](.test/multiplicity_tuple_array.svg)](.test/multiplicity_tuple_array.log)

```mu,multiplicity_tuple_array?mode=todo
r = Rect(width = 2mm, height = 2mm);

r.translate([(x=-4mm, y=-4mm), (x=-4mm, y=4mm), (x=4mm, y=-4mm), (x=4mm, y=4mm)]);
```
