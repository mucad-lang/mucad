
/// Circle definition with an offset.
///
/// Examples:
/// * With radius `r`: `Circle(r = 10.0mm);`
/// * With diameter `d`: `Circle(d = 5.0mm);`
pub sketch Circle(radius: Length, center = (x = 0.0mm, y = 0.0mm)) {
    init(diameter: Length, center = (x = 0.0mm, y = 0.0mm)) {
        radius = diameter / 2.0;
    }

    __mu::geo2d::Circle(radius)
        .__mu::ops::translate(x = center.x, y = center.y, z = 0.0mm);
}
