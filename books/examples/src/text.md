# Example: text

## Module: love

[![test](.test/text_love.svg)](.test/text_love.log)

```mu,text_love
// file: love.µcad
// Copyright © 2025 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::geo2d::*;
use std::ops::*;

sketch Heart(size: Length) {
    r = size / 3;
    Circle(d = size).translate(x = [r, -r], y = r) | Rect(size).rotate(45°);
}

#[color = std::color::GREEN]
Text("µcad", height = 80mm).center().translate(x = -110mm).extrude(4mm);

#[color = std::color::RED]
Heart(40mm).extrude(12mm);

#[color = std::color::BLUE]
Text("PTF", height = 80mm).center().translate(x = 90mm).extrude(4mm);

```

**2D Output**
    : ![None](.test/text_love-out.svg)

**3D Output**
    : ![None](.test/text_love-out.stl)

## Module: text_plate

[![test](.test/text_text_plate.svg)](.test/text_text_plate.log)

```mu,text_text_plate
// file: text_plate.µcad
// Copyright © 2025 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::geo2d::*;
use std::ops::*;
use std::math::*;

sketch TextPlate(text: String, height: Length = 30mm) {
    letters = Text(text, height).center();
    plate = RoundedRect(60mm, radius = 20mm);
    plate - letters
}

TextPlate("Hello µcad", 10mm)
    .extrude(2mm, 0°);

```

**2D Output**
    : ![None](.test/text_text_plate-out.svg)

**3D Output**
    : ![None](.test/text_text_plate-out.stl)
