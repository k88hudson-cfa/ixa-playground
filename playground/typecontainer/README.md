## Use case

We're a building plugin system for a framework that centers around
storing private data in a simulation context object and exposing
methods to other plugins, which requires initializing them in the right order.

We want to avoid explicit registration of dependencies; plugins should only be
initialized if they are used, which may in turn initialize other plugins.
For example, say a simulation author imports a module that defines plugins A, B,
and C. Plugin A's initializer calls Plugin B's API; Plugin B has no dependencies.
This means that plugin B should be initialized first, then plugin A;
plugin C should never be initialized.

The obvious way to implement this is to either use mutable references to the
shared context object or to implement interior mutability on a container for
plugin data (such as `RefCell<HashMap>`). However, this has the unfortunate
characteristic of requiring references to be exclusive across all plugins,
which is overly restrictive in a lazy initialization paradigm.

## Semantics

We would like to implement a type-indexed key-value store (`TypeContainer<K>`) which
has the following properties:

- The store is indexed by a type `K` where each `K` defines an associated value type `K::Value`
- Only one entry per `K` can be stored
- `K::Value` can be any type that implements `Any`
- Consumers may do the following with a *shared* reference to the store:
    - Register a new entry by providing an initializer for `K`
    - Get a shared reference to the `K::Value` for the entry of type `K`
- Consumers may do the following with an *exclusive* (mutable) reference to the store:
    - Get a mutable, exclusive reference to a `K::Value` for the entry of type `K`
- The public interface does *not* expose any of its interior mutability; consumers
  cannot mutate entries directly

## Implementation notes

Our implementations are based around storing values as `Box<dyn Any`> indexed
by `TypeId`. Every `K` implements a trait `Key` where `Key::Value` implements `Any`.
Interior mutability is implemented via `UnsafeCell`:

```rust

trait Key: 'static {
    type Value: Any;
}

pub struct TypeContainer {
    // InnerContainer maps TypeId -> Box<dyn Any>
    container: UnsafeCell<InnerContainer>,
}
impl InnerContainer {
    pub fn get<K: Key>(&self) -> Option<&K::Value> {
       // Find the entry for TypeId::of::<K>
       // If it exists, downcast Box<dyn Any> to &K::Value
    }

    pub fn get_mut<K: Key>(&mut self) -> Option<&mut K::Value> {
        // Same as get but with a mutable reference
    }

    pub fn try_insert<K: Key>(&mut self, value: K::Value) -> Result<()> {
        // Find the entry for TypeId::of::<K>, if it exists, return an error
        // If it does not exist, add a new entry of Box::new(value)
    }
}
```

