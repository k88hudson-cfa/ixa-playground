# Typed Enum


### Usage

<style>
    .code-compare td,
    .code-compare th {
        text-align: left;
        vertical-align: top;
    }

    .code-compare td pre,
    .code-compare td code {
        margin: 0;
        background-color: transparent;
    }
</style>

<table class="code-compare">
<thead>
<tr>

<th>Old</th>
<th>New</th>
</tr>
</thead>
<tbody>
<tr>
<td>

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InfectionStatusValue {
    Susceptible,
    Infected(InfectionData),
    Recovered,
}
property!(InfectionStatus, InfectionStatusValue);
```
</td>
<td>

```rust
property_enum!(InfectionStatus {
    Susceptible,
    Infected(InfectionData),
    Recovered,
});
```
</td>
</tr>
<tr>
<td>

```rust
let data = InfectionData { t: 1.0 };
context.set_property(
    person,
    InfectionStatus,
    InfectionStatusValue::Infected(data)
);
```
</td>
<td>

```rust
let data = InfectionData { t: 1.0 };
context.set_property(person, InfectionStatus, Infected(data));
```
</td>
</tr>
</tbody>
</table>

### Coercing between the marker types and enum variants

```rust
impl Context {
    fn set_property<P: Property, V: Into<P::Value>>(
        &mut self,
        person: PersonId,
        property: P,
        v: V,
    ) {
        let value: P::Value = v.into();
        println!("Setting property to value: {}", value);
    }
}
```

### Inference?

Annoyingly, the inference on numbers doesn't work if you switch the type
from `P::Value` to `impl Into<P::Value>`... any ideas?

```rust
context.set_property(person, Age, 30_u32);
```


### Playground

```rust
{{#rustdoc_include src/main.rs}}
```
