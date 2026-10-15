# Load an external module within MD test

[![test](.test/mod_external_root.svg)](.test/mod_external_root.log)

```mu,mod_external_root
mod external;
assert(std::debug::is_valid("external::a"));
```

[![test](.test/mod_external.svg)](.test/mod_external.log)

```mu,mod_external
// file: external.µcad
pub const a =1;
```
