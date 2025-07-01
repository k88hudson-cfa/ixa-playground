
## `TypeContainer`

### Use case

We are building a plugin system for a shared simulation context object
that avoids explicit registration of dependencies; plugins should only be
initialized if they are used, which may in turn may initialize other plugins.
For example, say a simulation author imports a module that defines plugins A, B,
and C. Plugin A's initializer calls Plugin B's API; Plugin B has no dependencies.
This means that plugin B should be initialized first, then plugin A;
plugin C should never be initialized.

Our current implementation uses mutable references to the
shared context object – we have also explored various approaches which implement
interior mutability on a container for plugins (like `RefCell<HashMap>`). However, this has the unfortunate characteristic of requiring references to be exclusive across all plugins,
which is overly restrictive in a lazy initialization paradigm.

### Semantics

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

## `VecCell`

### Use case

We want to store property values for entities in a simulation (e.g.,`Age` is a property of `Person`); properties may have a default value or initializer, and different
modules in a simulation need to be able to get, set, and query over entity properties. Simulations may have a very a large number of entities and their properties are
accessed often; the distribution of values varies depending on the property.

### Semantics

- Entities are assigned an index, which is a a `usize` starting at `0` that
  increments every time a new entity is added. Entities are never removed.
- Every property has a unique type `P`, which defines a value type `P:Value`
- `P:Value` must implement `Copy`
- Properties may define an initializer which returns a `P:Value`, and which
  gets access to a global context object
- Consumers may do the following with a *shared* reference:
    - Get a property given an entity's index; if the value exists, it should be
      returned (with `Copy`); otherwise, it should call the initializer
    - Query for all indexes matching a particular value
- Consumers may do the following with a *exclusive* (mutable) reference:
    - Set a property for a given entity's index
