# Context as Trait (Improved)

In the current version of Ixa, `Context` is a struct. This is natural to work
with in almost all cases *except* when writing a trait extension for it;
the annoyance is that even though this trait is designed only to be implemented
for `Context`, because you have to use a bunch of its methods you can't
write a default implementation (and therefore always have to write the definition
and implementation separately).

This is kind of not ideal when refactoring.

```rust
trait PeopleContextExt {
    fn get_person_property(&self, id: &PersonId) -> PersonProperty;
    fn set_person_property(&mut self, id: &PersonId, property: PersonProperty);
}
impl PeopleContextExt for Context {
    fn get_person_property(&self, id: &PersonId) -> PersonProperty {
        self.get_data::<PeoplePlugin>().get_person_property(id)
    }

    fn set_person_property(&mut self, id: &PersonId, property: PersonProperty) {
        self.get_data_mut::<PeoplePlugin>().set_person_property(id, property);
    }
}
```

## The solution

Instead of putting all the methods we need on `Context`'s, we can make it a *supertrait*
instead:

```rust
struct Context {}
// Methods that context extensions don't need
impl Context {
    fn new() -> Self {
        Context {}
    }
}
// Methods that context extensions need
trait PluginContext: PluginData {
    fn get_data<T: PluginData>(&self) -> &T;
    fn get_data_mut<T: PluginData>(&mut self) -> &mut T;
}

impl PluginContext for Context {
    fn get_data<T: PluginData>(&self) -> &T {
        // ...
    }

    fn get_data_mut<T: PluginData>(&mut self) -> &mut T {
        // ...
    }
}
```

Now we can implement the `PeopleContextExt` trait for any type that implements `PluginContext`.

Cool!!

```rust
trait PeopleContextExt: PluginContext {
    fn get_person_property(&self, id: &PersonId) -> PersonProperty {
        self.get_data::<PeoplePlugin>().get_person_property(id)
    }

    fn set_person_property(&mut self, id: &PersonId, property: PersonProperty) {
        self.get_data_mut::<PeoplePlugin>().set_person_property(id, property);
    }
}
```

## Avoiding impl PluginContext

The only issue with the above is that in some situations (such as for callbacks
that take a `Context`), it's not ideal to have to operate over an `impl PluginContext`,
or to have to import `PluginContext` everywhere (note this would be a breaking change
for users not using `ixa::prelude::*`).

The avoid this, we can leave `Context` as is, and instead have the trait mirror methods we want
to expose to plugins:

```rust
trait PluginContext {
    fn get_data<T: PluginData>(&self) -> &T;
    fn get_data_mut<T: PluginData>(&mut self) -> &mut T;
}
impl PluginContext Context {
    fn get_data<T: PluginData>(&self) -> &T {
        Context::get_data::<T>(self)
    }

    fn get_data_mut<T: PluginData>(&mut self) -> &mut T {
        Context::get_data_mut::<T>(self)
    }
}
```

That way:

* You can use `PluginContext` as a supertrait when defining traits that extend `Context`
* You can use `Context` directly everywhere else
* No need to import `PluginContext`, so this is fully backwards compatible.


### Playground

```rust
{{#rustdoc_include src/main.rs}}
```
