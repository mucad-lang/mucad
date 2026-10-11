
// Test statement for Source

//! Inner doc comment

#![code] // ok

sketch A() {} // ok
mod b {} // ok
mod c; // ok
fn d() {} // ok
use e; // ok
const F = 42; // ok
h = 42; // ok
prop foo: Length = 42mm; // error: lower::unexpected_statement
init() {} // error: lower::unexpected_statement
#![code] // error: lower::inner_attr_after_stmt

42; // ok


sketch B(a: Length) {
    sketch A() {} // error: lower::unexpected_statement

    mod b {} // error: lower::unexpected_statement

    mod c; // error: lower::unexpected_statement

    fn d() {} // ok
    use e; // ok
    const F = 42; // ok
    init() {a = 42mm;} // ok
    return 42; // error: lower::unexpected_statement
    #![g] // error: lower::inner_attr_after_stmt
    h = 42; // ok
    prop foo: Length = 42mm; // ok
    42; // ok
}


fn f() { // ok
    const A = 42; // ok
    sketch B() {} // error: lower::unexpected_statement

    init() {} // error: lower::unexpected_statement
}

mod module {
    prop foo: Length = 42mm; // error: lower::unexpected_statement
    init() {} // error: lower::unexpected_statement

    fn f() {} // ok
    const A = 42; // ok
    a = 42; // error: lower::unexpected_statement

    sketch A(foo: Integer) {
        init() {
            prop foo = 42; // error: lower::unexpected_statement
            foo = 42; // ok
        } // ok


    } // ok
}
