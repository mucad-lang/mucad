// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

/// A disc.
// <- comment.doc
#[color = "red"]
// <- attribute
// ^ attribute
//        ^ string
pub sketch Disc(radius: Length = 1mm) {
// <- keyword
//  ^ keyword
//         ^ type
//              ^ variable.parameter
//                      ^ type
//                               ^ number
    prop area = PI * radius;
//  ^ keyword
//       ^ property
//              ^ constant
    std::geo2d::Circle(r = radius).translate(x = 1mm);
//              ^ type
//                     ^ property
//                                 ^ function
    @input;
//  ^ variable.special
}
