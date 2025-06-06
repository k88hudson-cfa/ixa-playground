# Global Registration of Plugins

This example demonstrates how plugins can be registered globally via `ctor` and initialized lazily with a `OnceCell`.
Each plugin provides a value that is constructed on demand the first time it is accessed.

This setup enables plugin modularity without requiring manual registration in every context instance,
but without doing extra initialization for plugins that are not used.

Dependency resolution happens automatically as data plugins are accessed.

### Playground

```rust
{{#rustdoc_include src/main.rs}}
```
