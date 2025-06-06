# Settings Enum

This variation implements the Settings Context extension for a generic of `T: SettingsType`,
where we expect `T` to be an enum.

```rust
pub trait SettingContextExt<T: SettingType> {
    fn get_setting_property<P: SettingProperty>(&self, setting: SettingId<T>) -> P::Value;
    fn get_settings_members(&self, setting_id: SettingId<T>) -> Vec<PersonId>;
    fn get_itinerary(&self, person: PersonId) -> Vec<ItineraryEntry<T>>;
    fn get_random_setting(&self, person: PersonId) -> SettingId<T>;
}

// User code
enum MySettings {
    Home,
    Work
}
impl SettingContextExt<MySettings> for Context {}

```

### Playground

```rust
{{#rustdoc_include src/main.rs}}
```
