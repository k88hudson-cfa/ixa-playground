# Context as Trait

Instead of defining a concrete `Context` struct, this pattern
uses a **`Context` trait** that can be extended **in a single trait block**.

Instead of:

```rust
trait NumberExt {
    fn get_number(&self) -> u32;
}

impl NumberExt for Context {
    fn get_number(&self) -> u32 {
        self.get_data::<NumberPlugin>()
    }
}
```

You can write the implementation as a *default* implementation on the trait itself:

```rust
trait NumberExt: Context {
    fn get_number(&self) -> u32 {
        self.get_data::<NumberPlugin>()
    }
}
```

Dependencies and regular functions work slightly differently;

You must add any dependency context extensions to the trait definition:

```rust
trait NumberExt: Context + BoolPlugin {
    fn get_bool_as_number(&self) -> u32 {
        if self.get_data::<BoolPlugin>() {
            1
        } else {
            0
        }
    }
}
```

Regular functions can be implemented on a generic or on BaseContext (the struct)

```rust
pub fn do_stuff_with_numbers<T: NumberExt>(context: &T) {
    let number = context.get_number();
    println!("Number is: {}", number);
}

pub fn do_stuff_with_numbers_base(context: &BaseContext) {
    let number = context.get_number();
    println!("Number is: {}", number);
}
```


### Playground

```rust
{{#rustdoc_include src/main.rs}}
```
