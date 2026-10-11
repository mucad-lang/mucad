# Ambiguous Look Up for properties/locals and operations

[![test](.test/method_call.svg)](.test/method_call.log)

```mu,method_call
op scale() {}
sketch Sketch(scale: Scalar) { std::geo2d::Rect(size = scale * 40mm) }
Sketch(4.0);
```
