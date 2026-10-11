# Multiplicity

A core concept of µcad is called *Argument Multiplicity* which replaces *loops* known from imperative languages.

When working with multiplicities, each argument can be provided as an *array* of elements of a parameter's type.

Each list element will then be evaluated for each of the array's values.
This way, we can intuitively express a call that is executed for each argument variant.

The following example will produce 4 rectangles at different positions:

[![test](.test/multiplicity_arrays.svg)](.test/multiplicity_arrays.log)

```mu,multiplicity_arrays
r = std::geo2d::Rect(width = 2mm, height = 2mm);

r.std::ops::translate(x = [-4mm, 4mm], y = [-4mm, 4mm]);
```

Because in the above example `x` and `y` have two values each, the result
are four (2×2) calls:

[![test](.test/no_multiplicity.svg)](.test/no_multiplicity.log)

```mu,no_multiplicity
r = std::geo2d::Rect(width = 2mm, height = 2mm);

use std::ops::translate;
r.translate(x = -4mm, y = -4mm);
r.translate(x = -4mm, y = 4mm);
r.translate(x = 4mm, y = -4mm);
r.translate(x = 4mm, y = 4mm);
```

Normally, this would require two nested *for loops*.
As you may imagine, multiplicity can be a powerful tool.
