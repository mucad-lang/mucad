# Argument matching

[![test](.test/argument_match_short_id.svg)](.test/argument_match_short_id.log)

```mu,argument_match_short_id
fn f( x_param: Length, y_param: Length, z_param=0mm ) -> Volume  {
    x_param * y_param * z_param
}

f( x_param=1m, y_param=2m, z_param=3m );
f( x_p=1m, y_p=2m, z_p=3m );
f( x_p=1m, y_p=2m );
```

[![test](.test/argument_match_short_collision.svg)](.test/argument_match_short_collision.log)

```mu,argument_match_short_collision#fail
sketch S( width: Length ) {
    init( what: Length ) { width=what; }
}

S(w=10cm); // error: short form cannot be used here because of ambiguity
```

[![test](.test/argument_match_collision.svg)](.test/argument_match_collision.log)

```mu,argument_match_collision#fail
sketch S( width: Length ) {
    init( width: Length ) { width=width; }
}

S(width=10cm); // error: short form cannot be used here because of ambiguity
```

[![test](.test/argument_match_collision_init.svg)](.test/argument_match_collision_init.log)

```mu,argument_match_collision_init#fail
sketch S( width: Length, height: Length ) {
    init( size: Length ) { width=size; height=size; }
    init( size: Length ) { width=size; height=size; }
}

S(size=10cm); // error: short form cannot be used here because of ambiguity
```
