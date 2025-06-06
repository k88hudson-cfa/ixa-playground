mod context {
    use polonius_the_crab::prelude::*;
    use std::{
        any::{Any, TypeId},
        cell::{OnceCell, RefCell},
        collections::{HashMap, HashSet},
        sync::{LazyLock, Mutex},
    };

    pub static DATA_PLUGINS: LazyLock<Mutex<RefCell<HashSet<TypeId>>>> =
        LazyLock::new(|| Mutex::new(RefCell::new(HashSet::new())));

    pub fn add_plugin_to_registry<T: DataPlugin>() {
        DATA_PLUGINS
            .lock()
            .unwrap()
            .borrow_mut()
            .insert(TypeId::of::<T>());
    }

    pub trait Context {
        fn get_data<T: DataPlugin>(&self, _plugin: T) -> &T::DataContainer;
        fn get_data_mut<T: DataPlugin>(&mut self, _plugin: T) -> &mut T::DataContainer;
    }

    pub struct BaseContext {
        data_plugins: HashMap<TypeId, OnceCell<Box<dyn Any>>>,
    }

    impl BaseContext {
        pub fn new() -> Self {
            let mut context = BaseContext {
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
        fn do_stuff() {
            println!("Doing stuff in BaseContext");
        }
    }

    impl Context for BaseContext {
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
        fn init(context: &BaseContext) -> Self::DataContainer;
    }

    #[macro_export]
    macro_rules! define_data_plugin {
        ($data_plugin:ident, $data_container:ty, $init:expr) => {
            struct $data_plugin;

            impl $crate::context::DataPlugin for $data_plugin {
                type DataContainer = $data_container;

                fn init(context: &$crate::context::BaseContext) -> Self::DataContainer {
                    $init(context)
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
}

mod bool_plugin {
    use super::context::*;
    use super::define_data_plugin;

    define_data_plugin!(BoolPlugin, bool, |_context| false);

    pub trait BoolExt: Context {
        fn set_bool(&mut self, value: bool) {
            *self.get_data_mut(BoolPlugin) = value;
        }
        fn get_bool(&self) -> bool {
            *self.get_data(BoolPlugin)
        }
    }
    impl<T: Context + ?Sized> BoolExt for T {}
}

mod number_plugin {
    use crate::bool_plugin::BoolExt;
    use crate::{context::Context, define_data_plugin};

    define_data_plugin!(NumberPlugin, u32, |_context| 0);

    pub trait NumberExt: Context {
        fn set_number(&mut self, value: u32) {
            *self.get_data_mut(NumberPlugin) = value;
        }
        fn get_number(&self) -> u32 {
            *self.get_data(NumberPlugin)
        }
        fn get_bool_as_number(&self) -> u32 {
            if self.get_bool() { 1 } else { 0 }
        }
    }
    impl<T: Context + ?Sized> NumberExt for T {}
}

use bool_plugin::BoolExt;
use context::*;

use crate::number_plugin::NumberExt;

pub fn main() {
    // Example usage
    let mut context = BaseContext::new();
    assert!(!context.get_bool());
    assert_eq!(context.get_bool_as_number(), 0);
    assert_eq!(context.get_number(), 0);
    context.set_bool(true);
    assert!(context.get_bool());
    assert_eq!(context.get_bool_as_number(), 1);
    context.set_number(2);
    assert_eq!(context.get_number(), 2);
}

#[cfg(test)]
mod test {
    use super::context::{BaseContext, Context};
    use super::define_data_plugin;

    define_data_plugin!(A, usize, |context: &BaseContext| {
        println!("Initializing A");
        *context.get_data(B) + 1
    });

    define_data_plugin!(B, usize, |context: &BaseContext| {
        println!("Initializing B");
        *context.get_data(C) + 1
    });

    define_data_plugin!(C, usize, |_context| {
        println!("Initializing C");
        0
    });

    #[test]
    fn test() {
        let context = BaseContext::new();

        assert_eq!(*context.get_data(C), 0);
        assert_eq!(*context.get_data(A), 2);
        assert_eq!(*context.get_data(B), 1);
    }

    #[test]
    fn test_mut() {
        let mut context = BaseContext::new();

        *context.get_data_mut(C) = 1;
        assert_eq!(*context.get_data(A), 3);
        assert_eq!(*context.get_data(B), 2);
    }
}
