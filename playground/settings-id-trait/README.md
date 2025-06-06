# Settings ID Trait

This example shows how you can use a trait to support a specific AND generic
version of a setting ID:

```rust
pub trait SettingContextExt {
    // We know the type of setting
    fn get_settings_members<T: SettingType>(&self, setting_id: SettingId<T>) -> Vec<PersonId>;
    // We don't know the type of setting
    fn get_random_setting(&self, person: PersonId) -> Box<dyn AnySettingId>;
    fn get_itinerary(&self, person: PersonId) -> Vec<ItineraryEntry>;
    fn get_alpha(&self, person: PersonId, setting: impl AnySettingId) -> f64;
}
```

### Playground

```rust
{{#rustdoc_include src/main.rs}}
```
