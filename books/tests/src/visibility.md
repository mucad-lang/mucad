# Visibility

[![test](.test/visibility.svg)](.test/visibility.log)

```mu,visibility
mod my {
    mod mod_private {
        pub const_public = 1;
        const const_private = 1;
    }
    pub mod mod_public {
        pub const_public = 1;
        const const_private = 1;
    }
}

use __builtin::debug::assert_valid;
use __builtin::debug::assert_invalid;

assert_valid("my::mod_public::const_public");
assert_invalid("my::mod_public::const_private");
assert_invalid("my::private_public::const_public");
assert_invalid("my::private_public::const_private");
```
