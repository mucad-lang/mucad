# Symbols

[![test](.test/symbol_doublet.svg)](.test/symbol_doublet.log)

```mu,symbol_doublet?result=fail
mod foo {
    pub const A = 1mm;
}

fn foo() -> Length { // error: Ambiguous identifiers
    foo::A
}

foo();
```
