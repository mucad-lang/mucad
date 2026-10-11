# Code blocks

## Empty code block

```mu,test_empty
```

## Code block with failing test

```mu,test_fail#fail
Circle(r = 42mm);
```

## Code block with parameters

```mu,test_params(hires)
Rect(r = 42mm);
```

## Code block with test result and parameters

[![test](.test/test_params.svg)](.test/test_params.log)

```mu,test_result_params#fail(no_format)
Rect(r = 42mm);
```

## Multi-line Code block

[![test](.test/multi_line.svg)](.test/multi_line.log)

```mu,multi_line
use std::print;

print("Hello, µcad standard library!");
```
