# Primitive Types

## Bool

Boolean is the result type of boolean expressions, which may be `true` or `false`.

[![test](.test/types_primitive_bool.svg)](.test/types_primitive_bool.log)

```mu,types_primitive_bool
std::assert(true != false);
```

Boolean values can be combined with `or` and `and` [operators](../expressions/operators.md):

[![test](.test/boolean.svg)](.test/boolean.log)

```mu,boolean
std::assert([true or false, true]);
std::assert(true and false == false);

std::assert((4 == 4) == true);
std::assert((4 == 5) == false);
std::assert((4 == 5 or 4 == 4) == true);
std::assert((4 == 5 and 4 == 4) == false);
```

## Integer

The type `Integer` contains an integer number.

[![test](.test/types_primitive_integer.svg)](.test/types_primitive_integer.log)

```mu,types_primitive_integer
i = 3;
```

## String

*Strings* are mostly used for rendering text.

[![test](.test/types_primitive_string.svg)](.test/types_primitive_string.log)

```mu,types_primitive_string
text = "Hello µcad!";
std::assert(std::string::len(text) == 11);

// logging
std::print(text);
```

## Matrix

Matrix types are built-in types and can be defined as:

- `Matrix2`: A 2x2 matrix, commonly used for 2D rotations.
- `Matrix3`: A 3x3 matrix, commonly used for rotations.
- `Matrix4`: A 4x4 matrix, commonly used for affine transformations.

*Note: Currently, matrices are used only internally and cannot be instantiated from µcad code.*
