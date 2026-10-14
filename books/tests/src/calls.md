# Calls

[![test](.test/call_same_parameter_name.svg)](.test/call_same_parameter_name.log)

```mu,call_same_parameter_name?result=fail
use std::geo2d::*;
use std::ops::*;

Rect(w = 10mm, w = 10mm);
```
