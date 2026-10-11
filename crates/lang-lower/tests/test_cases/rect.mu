pub sketch Rect(width: Length, height: Length, x: Length, y: Length) {
    init(width: Length, height: Length) {
        x = -width / 2.0; // center x
        y = -height / 2.0; // center y
    }

    init(size: Length) {
        width = size;
        height = size;
        x = -size / 2.0; // center x
        y = -size / 2.0; // center y
    }

    __mu::geo2d::Rect(width, height, x, y);
}
