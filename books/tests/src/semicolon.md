# Usage of semicolon with Workbenches

[![test](.test/operation_with_body.svg)](.test/operation_with_body.log)

```mu,operation_with_body
use std::geo2d::Circle;
use std::ops::translate;

{ // op with body
    Circle(radius = 5mm);
}.translate(y=[-34mm/2 , 34mm/2]);
```

[![test](.test/operation_no_body.svg)](.test/operation_no_body.log)

```mu,operation_no_body
use std::geo2d::Circle;
use std::ops::translate;

 // op without body
Circle( radius = 5mm )
    .translate(y = [-34mm/2 , 34mm/2]);
```

[![test](.test/sketch_missing_semicolon.svg)](.test/sketch_missing_semicolon.log)

```mu,sketch_missing_semicolon
use std::geo2d::Circle;
use std::ops::translate;

{
    Circle(radius = 5mm) // missing semicolon is ok.
}.translate(y=[-34mm/2 , 34mm/2]);
```

[![test](.test/sketch_with_empty_body.svg)](.test/sketch_with_empty_body.log)

```mu,sketch_with_empty_body?mode=todo
{}.translate(x = 5mm);
```

[![test](.test/sketch_with_body.svg)](.test/sketch_with_body.log)

```mu,sketch_with_body?result=fail
use std::geo2d::Circle;

Circle(radius = 2mm) { Circle(radius = 1mm); } // parse_error: sketch with body
```

[![test](.test/empty_op.svg)](.test/empty_op.log)

```mu,empty_op?result=fail
translate(x = 3.0mm); // error: Cannot call operation without workpiece. 
```

[![test](.test/group.svg)](.test/group.log)

```mu,group
use std::geo2d::Circle;
use std::ops::translate;

// group
{ 
    Circle(radius = 1mm); 
    Circle(radius = 2mm); 
}
```

[![test](.test/group_assignment.svg)](.test/group_assignment.log)

```mu,group_assignment
use std::geo2d::Circle;
use std::ops::translate;

// assignment + group
a = { 
    Circle(radius = 1mm); 
    Circle(radius = 2mm); 
};
```
