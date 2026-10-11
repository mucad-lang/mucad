sketch A(d: Length) {
    Circle(d);
}

sketch A(a: Length, b: Length) {
    Circle(d = a + b);
}

sketch B(d: Length) {
    prop test = 3.0;
    Circle(d);
}

sketch C(d: Length) {
    prop test = 3.0;

    Circle(d);
}

/// A doc comment
/// Another doc comment
#[some, attributes]
#[color = RED]
#[export("to_file.svg")]
sketch D(
    a: Length,
    b: Length,
    c: Length,
    d: Length,
    e: Length,
) {
    prop test = 3.0;

    init(a: Length) {}

    Circle(d);
}
