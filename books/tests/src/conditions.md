# Conditions

[![test](.test/condition_return.svg)](.test/condition_return.log)

```mu,condition_return
fn f(x: Integer) {
    if x == 5 or x == 4 {
        return "match";
    } else if x > 0 and x < 4 {
        return "no match";
    } else {
        return "invalid";
    }
}

print(f(5));  // prints "match"
```
