
sketch A(a: Length) {
    prop b = 10mm; // error: lower::unexpected_statement

    init(c: Length) {
        a = c;
    }

    prop d = 20mm; // error: lower::unexpected_statement

    init(e: Length) {
        a = e;
    }

    prop f = 30mm; // ok
}
