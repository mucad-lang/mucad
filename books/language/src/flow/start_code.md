# Start Code

A µcad program consists of one or multiple files.

Additional files can be added by the [`mod` statement](../structure/modules/file_modules.md)
or by adding library paths of external modules with command line options.

Currently[^no_project], every µcad program starts with the original input file
that was given at running `microcad`.

The **top level code** within this file is where microcad starts processing top to bottom.

[![test](.test/start.svg)](.test/start.log)

```mu,start
// 1. Start code begins here.
use std::geo2d::*;

// 2. This is a module definition. It defines 'RADIUS' but doesn't generate any geometry.
mod my_inner {
    pub const RADIUS = 10mm;
}

// 3. Start code continues here.
Circle(radius = my_inner::RADIUS);
```

[^no_project]: In future µcad will get a package management and will have projects and toml files.
