use ctor::ctor;
use polonius_the_crab::prelude::*;
use std::{
    any::{Any, TypeId},
    cell::{OnceCell, RefCell},
    collections::{HashMap, HashSet},
    sync::{LazyLock, Mutex},
};

pub static DATA_PLUGINS: LazyLock<Mutex<RefCell<HashSet<TypeId>>>> =
    LazyLock::new(|| Mutex::new(RefCell::new(HashSet::new())));
pub struct Context {
    data_plugins: HashMap<TypeId, OnceCell<Box<dyn Any>>>,
}

pub fn add_plugin_to_registry<T: DataPlugin>() {
    DATA_PLUGINS
        .lock()
        .unwrap()
        .borrow_mut()
        .insert(TypeId::of::<T>());
}

impl Context {
    pub fn new() -> Self {
        let mut context = Context {
            data_plugins: HashMap::new(),
        };
        // Register all data plugins from global list
        for plugin_type_id in DATA_PLUGINS.lock().unwrap().borrow().iter() {
            context
                .data_plugins
                .insert(*plugin_type_id, OnceCell::new());
        }
        context
    }

    pub fn get_data<T: DataPlugin>(&self, _plugin: T) -> &T::Value {
        let type_id = TypeId::of::<T>();
        self.data_plugins
            .get(&type_id)
            .expect("Data plugin not registered")
            .get_or_init(|| Box::new(T::init(self)))
            .downcast_ref::<T::Value>()
            .unwrap()
    }

    pub fn get_data_mut<T: DataPlugin>(&mut self, _plugin: T) -> &mut T::Value {
        let mut self_shadow = self;
        let type_id = TypeId::of::<T>();
        // If the data plugin is already initialized return mutable reference
        // Use polonius to address borrow checker limitations
        polonius!(|self_shadow| -> &'polonius mut T::Value {
            let cell = self_shadow
                .data_plugins
                .get_mut(&type_id)
                .expect("Data plugin not registered");
            if let Some(any) = cell.get_mut() {
                polonius_return!(any.downcast_mut::<T::Value>().unwrap());
            }
        });
        // Initialize the data plugin
        let data = T::init(self_shadow);
        let cell = self_shadow.data_plugins.get_mut(&type_id).unwrap();
        let _ = cell.set(Box::new(data));
        cell.get_mut().unwrap().downcast_mut::<T::Value>().unwrap()
    }
}

pub trait DataPlugin: 'static {
    type Value;

    fn init(context: &Context) -> Self::Value;
}

struct MyPlugin;
impl DataPlugin for MyPlugin {
    type Value = u32;

    fn init(_context: &Context) -> Self::Value {
        println!("Initializing MyPlugin");
        42
    }
}
#[ctor]
fn register_my_plugin() {
    add_plugin_to_registry::<MyPlugin>()
}

pub fn main() {
    // Example usage
    let context = Context::new();
    assert_eq!(*context.get_data(MyPlugin), 42);
    println!("{}", context.data_plugins.len());
}

#[cfg(test)]
mod test {
    use ctor::ctor;

    use super::{Context, DataPlugin, add_plugin_to_registry};

    struct A;
    #[ctor]
    fn register_a() {
        add_plugin_to_registry::<A>()
    }

    impl DataPlugin for A {
        type Value = usize;

        fn init(context: &super::Context) -> Self::Value {
            println!("Initializing A");
            *context.get_data(B) + 1
        }
    }

    struct B;
    #[ctor]
    fn register_b() {
        add_plugin_to_registry::<B>()
    }
    impl DataPlugin for B {
        type Value = usize;

        fn init(context: &super::Context) -> Self::Value {
            println!("Initializing B");
            *context.get_data(C) + 1
        }
    }

    struct C;
    #[ctor]
    fn register_c() {
        add_plugin_to_registry::<C>()
    }
    impl DataPlugin for C {
        type Value = usize;

        fn init(_context: &super::Context) -> Self::Value {
            println!("Initializing C");
            0
        }
    }

    #[test]
    fn test() {
        let context = Context::new();
        println!("{}", context.data_plugins.len());

        assert_eq!(*context.get_data(C), 0);
        assert_eq!(*context.get_data(A), 2);
        assert_eq!(*context.get_data(B), 1);
    }

    #[test]
    fn test_mut() {
        let mut context = Context::new();
        println!("{}", context.data_plugins.len());

        *context.get_data_mut(C) = 1;
        assert_eq!(*context.get_data(A), 3);
        assert_eq!(*context.get_data(B), 2);
    }
}
