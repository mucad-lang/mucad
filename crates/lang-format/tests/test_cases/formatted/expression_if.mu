sketch MySketch(x: Integer) {
    if x > 0 {
        Circle(radius = 5mm)
    } else if x < 0 {
        Rect(10mm)
    } else {
        Hexagon(5mm)
    }
}
