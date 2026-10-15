# Tuple Aliases

Some named tuples are used frequently and thus they have aliases: `Color`, `Vec2` and `Vec3`.
This feature is also called *named-tuple duck-typing*.

## Color

A named tuple with the fields `r`, `g`, `b` and `a` will be treated as a color with red, green, blue and alpha channel.
You can use the type alias `Color` to deal with color values.

[![test](.test/types_named_tuple_color.svg)](.test/types_named_tuple_color.log)

```mu,types_named_tuple_color
color: Color = (r = 100%, g = 50%, b = 25%, a = 100%);

std::assert_eq(color.r, 100%);
```

## Vec2

A named tuple with the fields `x` and `y` will be treated as a two-dimensional vector.
You can use the type alias `Vec2`.

[![test](.test/types_named_tuple_vec2.svg)](.test/types_named_tuple_vec2.log)

```mu,types_named_tuple_vec2
v: Vec2 = (x = 2.0, y = 3.0);

std::assert_eq(v.x + v.y, 5.0);
```

## Vec3

A named tuple with the fields `x`, `y` and `z` will be treated as a three-dimensional vector.
You can use the type alias `Vec3`.

[![test](.test/types_named_tuple_vec3.svg)](.test/types_named_tuple_vec3.log)

```mu,types_named_tuple_vec3
v: Vec3 = (x = 2.0, y = 3.0, z = 4.0);

std::assert_eq(v.x + v.y + v.z, 9.0);
```
