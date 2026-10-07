const R = 4mm;

sketch Disc(radius: Length) {
    __mu::geo2d::Circle(radius = radius);
}

sketch Discs(gap: Length) {
    init(diameter: Length) {
        gap = diameter / 2;
    }
    d = Disc(R);
    d;
    Disc(gap);
}
