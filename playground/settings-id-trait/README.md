# Settings ID Trait

This example shows how you can write functions against a specific or a generic
version of a SettingId:

The basic idea is that you use a `SettingId<T>` when you know the setting type
(e.g.,`SettingId<Home>`), and a trait (`AnySettingId`) when you don't:

```rust
pub trait SettingContextExt {
    // We know the type of setting
    fn get_members<T: SettingType>(&self, id: SettingId<T>) -> Vec<PersonId>;
    // We don't know the type of setting
    fn get_random_setting(&self, person: PersonId) -> Box<dyn AnySettingId>;
    // We know the type of setting, but we want this to work with ANY setting type
    fn get_alpha(&self, person: PersonId, setting: impl AnySettingId) -> f64;
}
```

### Playground

```rust
{{#rustdoc_include src/main.rs}}
```
