# Attributes

Attributes are syntax elements used to attach **metadata** to models or **control output** during rendering and exporting.
While they do not alter the geometry of the model tree itself, they provide essential instructions for viewers and exporters.

## Attribute Categories

| Type                                   | Syntax Example          | Purpose                                                             |
| -------------------------------------- | ----------------------- | ------------------------------------------------------------------- |
| [**Metadata**](metadata_attributes.md) | `#[color = "red"]`      | Attaches key-value pairs for organizational or aesthetic metadata.  |
| [**Command**](command_attributes.md)   | `#[export("mesh.stl")]` | Triggers specific actions, like file generation or render settings. |

---

## Outer vs. Inner Attributes

The primary difference lies in the **scope** of what the attribute affects.

### Outer Attributes (`#[...]`)

Outer attributes are placed **outside** and **before** a statement. They apply to the specific node or block immediately following them.

[![test](.test/outer_attributes.svg)](.test/outer_attributes.log)

```mu,outer_attributes
#[color = "#FF0000"]
{
    std::geo2d::Circle(r = 20mm);
    std::geo2d::Rect(size = 20mm).std::ops::translate(x = 20mm);
}
```

### Inner Attributes (`#![...]`)

Inner attributes are separate statements **inside** a block or source file.
They apply to the "parent" model they reside in, affecting everything within that scope.
The following code is semantically equivalent to the outer attribute example above:

[![test](.test/inner_attributes.svg)](.test/inner_attributes.log)

```mu,inner_attributes
{
    #![color = "#FF0000"]

    std::geo2d::Circle(r = 20mm);
    std::geo2d::Rect(size = 20mm).std::ops::translate(x = 20mm);
}
```

> **Pro Tip:** Inner attributes are particularly useful at the very top of a file to set global parameters without needing to wrap the entire script in a group body.
