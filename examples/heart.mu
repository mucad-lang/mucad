// LOVE

sketch Heart(size: Length) {
    r = size / 3;
    Circle(d = size).translate(x = [r, -r], y = r) | Rect(size * 99.8%).rotate(45°);
}

Heart(50mm).extrude(10mm);
