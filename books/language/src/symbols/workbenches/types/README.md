# Workbench Types

*Workbenches* come in three flavors:

| Type                          | Keyword  | Input Model | Output Model |
| ----------------------------- | :------- | :---------: | :----------: |
| [*parts*](parts.md)           | `part`   |   *none*    |      3D      |
| [*sketches*](sketches.md)     | `sketch` |   *none*    |      2D      |
| [*operations*](operations.md) | `op`     |  2D or 3D   |   2D or 3D   |

Mostly you may start directly with `part` or with a `sketch` which you then
operate (with an `op`) into a `part`.
