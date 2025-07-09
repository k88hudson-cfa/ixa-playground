use polonius_the_crab::prelude::*;
use std::{
    any::{Any, TypeId},
    cell::{OnceCell, RefCell},
    collections::{HashMap, HashSet},
    sync::{LazyLock, Mutex},
};

static DATA_PLUGINS: LazyLock<Mutex<RefCell<HashSet<TypeId>>>> =
    LazyLock::new(|| Mutex::new(RefCell::new(HashSet::new())));

pub fn add_plugin_to_registry<T: DataPlugin>() {
    DATA_PLUGINS
        .lock()
        .unwrap()
        .borrow_mut()
        .insert(TypeId::of::<T>());
}

pub fn get_plugin_ids() -> Vec<TypeId> {
    DATA_PLUGINS
        .lock()
        .unwrap()
        .borrow()
        .iter()
        .copied()
        .collect::<Vec<_>>()
}

pub trait PluginContext {
    fn get_data<T: DataPlugin>(&self, _plugin: T) -> &T::DataContainer;
    fn get_data_mut<T: DataPlugin>(&mut self, _plugin: T) -> &mut T::DataContainer;
}

pub struct Context {
    data_plugins: HashMap<TypeId, OnceCell<Box<dyn Any>>>,
}

impl Context {
    pub fn new() -> Self {
        let mut context = Context {
            data_plugins: HashMap::new(),
        };
        // Register all data plugins from global list
        for plugin_type_id in get_plugin_ids() {
            context.data_plugins.insert(plugin_type_id, OnceCell::new());
        }
        context
    }
}

impl PluginContext for Context {
    fn get_data<T: DataPlugin>(&self, _plugin: T) -> &T::DataContainer {
        let type_id = TypeId::of::<T>();
        self.data_plugins
            .get(&type_id)
            .expect("Data plugin not registered")
            .get_or_init(|| Box::new(T::init(self)))
            .downcast_ref::<T::DataContainer>()
            .unwrap()
    }

    fn get_data_mut<T: DataPlugin>(&mut self, _plugin: T) -> &mut T::DataContainer {
        let mut self_shadow = self;
        let type_id = TypeId::of::<T>();
        // If the data plugin is already initialized return mutable reference
        // Use polonius to address borrow checker limitations
        polonius!(|self_shadow| -> &'polonius mut T::DataContainer {
            let cell = self_shadow
                .data_plugins
                .get_mut(&type_id)
                .expect("Data plugin not registered");
            if let Some(any) = cell.get_mut() {
                polonius_return!(any.downcast_mut::<T::DataContainer>().unwrap());
            }
        });
        // Initialize the data plugin
        let data = T::init(self_shadow);
        let cell = self_shadow.data_plugins.get_mut(&type_id).unwrap();
        let _ = cell.set(Box::new(data));
        cell.get_mut()
            .unwrap()
            .downcast_mut::<T::DataContainer>()
            .unwrap()
    }
}

pub trait DataPlugin: 'static {
    type DataContainer;

    // This should be context: &impl Context
    fn init(context: &impl PluginContext) -> Self::DataContainer;
}

#[macro_export]
macro_rules! define_data_plugin {
    ($data_plugin:ident, $data_container:ty, |$ctx:ident| $body:expr) => {
        struct $data_plugin;

        impl $crate::context::DataPlugin for $data_plugin {
            type DataContainer = $data_container;

            fn init($ctx: &impl PluginContext) -> Self::DataContainer {
                $body
            }
        }

        paste::paste! {
            #[ctor::ctor]
            fn [<_register_plugin_$data_plugin:snake>]() {
                $crate::context::add_plugin_to_registry::<$data_plugin>()
            }
        }
    };
}

#[cfg(test)]
mod test {
    use super::*;

    define_data_plugin!(A, usize, |context| {
        println!("Initializing A");
        *context.get_data(B) + 1
    });

    define_data_plugin!(B, usize, |context| {
        println!("Initializing B");
        *context.get_data(C) + 1
    });

    define_data_plugin!(C, usize, |_context| {
        println!("Initializing C");
        0
    });

    #[test]
    fn test() {
        let context = Context::new();

        assert_eq!(*context.get_data(C), 0);
        assert_eq!(*context.get_data(A), 2);
        assert_eq!(*context.get_data(B), 1);
    }

    #[test]
    fn test_mut() {
        let mut context = Context::new();

        *context.get_data_mut(C) = 1;
        assert_eq!(*context.get_data(A), 3);
        assert_eq!(*context.get_data(B), 2);
    }
}
