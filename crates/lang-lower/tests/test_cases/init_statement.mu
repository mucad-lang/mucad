sketch A(a: Length) {
    init(b: Length) {
        a = b; // ok
    }

    init(c: Length) { // error: lower::input_not_initalized

    }

    init(d: Length) {
        a = 2.0 * b; // ok
    }

    init(e: Length) {
        f = 2.0; // error: lower::not_an_input_property
        a = 2.0 * b; // ok
    }

    init(a: Length) { // warning: lower::duplicated_default_initializer
    }
}
