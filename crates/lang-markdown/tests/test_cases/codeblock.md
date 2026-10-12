# Code blocks

## Empty code block

```mu,test_empty
```

## Code block with failing test

```mu,test_fail
Circle(r = 42mm);
```

## Code block with parameters

```mu,test_params?resolution=hires
Rect(r = 42mm);
```

## Code block with test result and parameters

```mu,test_result_params?fail&no_format
Rect(r = 42mm);
```

## Multi-line Code block

```mu,multi_line
use std::print;

print("Hello, µcad standard library!");
```
