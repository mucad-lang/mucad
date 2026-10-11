pub sketch Known(radius: Length) {
    __mu::geo2d::Circle(radius = radius);
}

pub sketch Missing(radius: Length) {
    __mu::geo2d::Unknown(radius = radius);
}
