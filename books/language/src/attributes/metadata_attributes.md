# Metadata attributes

Metadata attributes allow you to store and retrieve arbitrary key-value pairs.
Each attribute name must be unique within its scope to avoid conflicts.
This metadata is used by viewers for rendering or by exporters when writing output to a file.

## Key Properties

* **Non-Destructive**: Metadata does not change the mathematical definition of the geometry, only how it is processed or displayed.
* **Inheritance**: Depending on the exporter and renderer, metadata on a model may be inherited by its children unless overridden.

## `color` attribute

The `color` attribute attaches a color to a model node.
In a viewer and for some exporter, the model will be rendered in the specified color.

[![test](.test/attributes_color.svg)](.test/attributes_color.log)

```mu,attributes_color
#[color = "#FFFFFF"]
c = std::geo2d::Circle(r = 42.0mm);

std::assert(c#color == (r = 1.0, g = 1.0, b = 1.0, a = 1.0));
```

You can access the color property using the `#` accessor.

## `resolution` attribute

The resolution attribute defines the rendering fidelity of the model.
It determines how a smooth mathematical curve is sampled and converted into discrete segments or triangle.

* **Default**: Typically 0.1mm.
* **Percentage**: Using a percentage scales the fidelity relative to the default. Note that **higher resolution** means a **smaller step size**:
  * `100%` = `0.1mm` (Default)
  * `200% =`0.05mm` (Finer detail)
  * `50% = 0.2mm` (Coarser detail)

This means the circle in the example below will be rendered with a resolution `0.05mm`.

[![test](.test/attributes_precision.svg)](.test/attributes_precision.log)

```mu,attributes_precision
#[resolution = 200%]
c = std::geo2d::Circle(r = 42.0mm);

std::assert(c#resolution == 200%);
```
