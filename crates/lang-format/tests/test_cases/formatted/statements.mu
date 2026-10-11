use foo;
use bar::foo as baz;

use bar::*;

//! inner doc comments

#![inner, attribute(), works = true]

mod foo {}

mod foo;

mod nested {
    const FOO = 42mm;

    fn function() {}

    fn another_function(test: Length) -> Length { test }

    mod another_mod {}
}
