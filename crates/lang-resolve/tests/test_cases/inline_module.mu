// Testing inline module 

/// A constant
pub const A = "Foo";

/// This inline module has documentation 
pub mod b {
    pub const C = 42;

    pub mod d {
        const E = 42mm;
    }

    // A private module
    mod f {
        const G = 42.0;
    }
}
