# Settings Trait (Flat)

This is a variation of the Settings Trait example,
where settings are stored internally as the same entity (`SettingEntity`),
and `Setting<T>` implements an API on top of `EntityId<SettingEntity>`:

The main difference here is how data/indexing of properties are internally
stored; each setting instance gets a unique index across all settings, and
enumerating over settings of a specific type would need to be implemented
separately.

```rust
  pub struct SettingEntity;
    impl Entity for SettingEntity {}

    pub trait SettingType: 'static {}

    pub struct Setting<T: SettingType> {
        _phantom: std::marker::PhantomData<T>,
        pub entity_id: EntityId<SettingEntity>,
    }
```

### Playground

```rust
{{#rustdoc_include src/main.rs}}
```
