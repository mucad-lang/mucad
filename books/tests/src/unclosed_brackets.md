# Brackets of various kinds that aren't closed

[![test](.test/unclosed_tuple.svg)](.test/unclosed_tuple.log)

```mu,unclosed_tuple#fail
a = (1, 2, 3;
```

[![test](.test/unclosed_bracket_expr.svg)](.test/unclosed_bracket_expr.log)

```mu,unclosed_bracket_expr#fail
a = (1 + 2;
```

[![test](.test/unclosed_code_block.svg)](.test/unclosed_code_block.log)

```mu,unclosed_code_block#fail
a = {
    1 + 2;
```
