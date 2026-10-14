# Quantity Types

The following *quantity types* are [currently](https://codeberg.org/microcad/microcad/issues/76) supported in µcad:

| Type      | Metric Units                                | Imperial Units                 |
| --------- | ------------------------------------------- | ------------------------------ |
| `Scalar`  | `%` *or none*                               | -                              |
| `Length`  | `µm`, `mm`, `cm`, `m`                       | `in` or `"`, `ft` or `'`, `yd` |
| `Angle`   | `°` or `deg`, `grad`, `turn`,`rad`          |                                |
| `Area`    | `µm²`,`mm²`,`cm²`,`m²`                      | `in²`, `ft²` , `yd²`           |
| `Volume`  | `µm³`, `mm³`,`cm³`,`m³`,`ml`,`cl`,`l`, `µl` | `in³`, `ft³` , `yd³`           |
| `Density` | `g/mm³`                                     | -                              |
| `Weight`  | `g`, `kg`                                   | `lb`, `oz`                     |

> [!TIP]
> Special characters like `µ`, `°`, `²`, and  `³` have ASCII[^ascii] replacements
> you may use if your keyboard misses these letters
> (see [this appendix](../../appendix/special_characters.md) for more information).

[^ascii]: see [wikipedia about ASCII](https://de.wikipedia.org/wiki/American_Standard_Code_for_Information_Interchange)

## Scalar

The type `Scalar` contains a floating number (without any *unit*) and must be written with at least one decimal place (or in percent).

[![test](.test/types_quantity_scalar.svg)](.test/types_quantity_scalar.log)

```mu,types_quantity_scalar
zero = 0;
pi = 3.1415;
percent = 55%;
```

## Length

`Length` is used to describe a concrete length.

[![test](.test/types_quantity_length.svg)](.test/types_quantity_length.log)

```mu,types_quantity_length
millimeters = 1000mm;
centimeters = 100cm;
meters = 1m;
inches = 39.37007874015748in;

std::assert(std::list::all_equal([millimeters, centimeters, meters, inches]));
```

## Angle

Angles are used with rotations and in constrains when proving measures.

[![test](.test/types_quantity_angle.svg)](.test/types_quantity_angle.log)

```mu,types_quantity_angle
pi = std::math::PI;
radian = 1rad * pi;
degree = 180°;
degree_ = 180deg;
grad = 200grad;
turn = 0.5turns;

std::assert(std::list::all_equal([degree, degree_, grad, turn, radian]));
```

## Area

An `Area` is a two-dimensional quantity. It is the result when multiplying two `Length`s.

[![test](.test/types_quantity_area.svg)](.test/types_quantity_area.log)

```mu,types_quantity_area
a = 3cm;
b = 2cm;
area = a * b;
std::assert(area == 6cm²);
```

Here is an example of how to use different areal units:

[![test](.test/types_quantity_area_units.svg)](.test/types_quantity_area_units.log)

```mu,types_quantity_area_units
square_millimeter = 100000mm²;
square_centimeter = 1000cm2;
square_meter = 0.1m²;
square_inch = 155in²;

std::assert(square_millimeter == square_centimeter);
```

## Volume

A `Volume` is a three-dimensional quantity. It is the result when multiplying three `Length`s.

[![test](.test/types_quantity_volume.svg)](.test/types_quantity_volume.log)

```mu,types_quantity_volume
a = 3mm;
b = 2mm;
c = 4mm;

volume = a * b * c;

std::assert(volume == 24mm³);
```

Here is an example for units:

[![test](.test/types_quantity_volume_units.svg)](.test/types_quantity_volume_units.log)

```mu,types_quantity_volume_units
cubic_millimeter = 1000000.0mm³;
cubic_centimeter = 100.0cl;
cubic_meter = 0.001m3;
cubic_inch = 61.0237in³;
liter = 1.0l;
centiliter = 100.0cl;
milliliter = 1000.0ml;

std::assert(std::list::all_equal([
        cubic_millimeter,
        cubic_centimeter,
        cubic_meter,
        centiliter,
        milliliter,
    ]));
```

## Density

Currently `Density` has not specific use in µcad and only can use unit `g/mm³`.

[![test](.test/types_quantity_density_units.svg)](.test/types_quantity_density_units.log)

```mu,types_quantity_density_units
gramm_per_square_centimeters = 19.302g/mm³;
```

## Weight

Weights can be calculated by applying volumes to materials.

[![test](.test/types_quantity_weight.svg)](.test/types_quantity_weight.log)

```mu,types_quantity_weight
gram = 1000.0g;
kilogram = 1.0kg;
pound = 2.204623lb;

std::assert(gram == kilogram);
```
