# Settings Trait

In this example, each setting type (home, work, etc.) is unique entity, identified
as `Setting<Home>`:

```rust
pub struct Setting<T: SettingType> {
    _phantom: std::marker::PhantomData<T>,
}
impl<T: SettingType> Entity for Setting<T> {}

// Example definition
struct Work;
impl SettingType for Work {}
type WorkId = EntityId<Setting<Work>>;
```

In scenarios where we want to restrict functionality to a specific setting type,
the entity identifier type is `EntityId<Setting<T>>`.

In order to support operating over settings of different types generically, we
implement a `SettingId` trait:

```rust
pub trait SettingId {
    fn get_id(&self) -> usize;
    fn get_type_id(&self) -> TypeId;
}
```

This can be used with an internal type_id-based API in order to find corresponding property
values etc. For example, we can implement a "get setting property" to look for the property
value pair corresponding to any given setting type:

```rust
// Setting Context Extension
fn get_setting_property<P: EntityProperty>(&self, setting: Box<dyn SettingId>) -> P::Value {
    self.get_property_by_type_id::<P>(setting.get_type_id(), setting.get_id())
}

// Get a random setting and retrieve a shared property
let setting = context.get_random_setting(PersonId::new(1));
context.get_setting_property::<Alpha>(setting);
```

### Playground

```rust
{{#rustdoc_include src/main.rs}}
```
