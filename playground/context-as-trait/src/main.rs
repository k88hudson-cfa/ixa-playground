#![allow(unused_variables)]
#![allow(dead_code)]

mod context {
    use std::{
        any::{Any, TypeId},
        collections::HashMap,
    };

    pub struct Context {
        data_containers: HashMap<TypeId, Box<dyn Any>>,
        plans: Vec<Box<dyn FnOnce(&mut Self)>>,
    }

    impl Context {
        pub fn new() -> Self {
            Context {
                data_containers: HashMap::new(),
                plans: Vec::new(),
            }
        }
        pub fn execute(&mut self) {
            println!("Executing!");
            while let Some(plan) = self.plans.pop() {
                plan(self);
            }
        }
        fn plugin_data<P: Plugin>(&self) -> &P::DataContainer {
            self.data_containers
                .get(&TypeId::of::<P>())
                .and_then(|data| data.downcast_ref::<P::DataContainer>())
                .expect("Failed to downcast data container")
        }
        fn plugin_data_mut<P: Plugin>(&mut self) -> &mut P::DataContainer {
            self.data_containers
                .entry(TypeId::of::<P>())
                .or_insert_with(|| Box::new(P::default()))
                .downcast_mut::<P::DataContainer>()
                .expect("Failed to downcast data container")
        }
        fn add_plan(&mut self, time: f64, plan: impl FnOnce(&mut Self) + 'static) {
            self.plans.push(Box::new(plan));
        }
    }

    pub trait Plugin: 'static {
        type DataContainer;
        // TODO: Make this init of &Context
        fn default() -> Self::DataContainer;
    }

    pub trait PluginContext {
        fn plugin_data<P: Plugin>(&self) -> &P::DataContainer;
        fn plugin_data_mut<P: Plugin>(&mut self) -> &mut P::DataContainer;

        fn add_plan(&mut self, time: f64, plan: impl FnOnce(&mut Self) + 'static);
    }
    impl PluginContext for Context {
        fn plugin_data<P: Plugin>(&self) -> &P::DataContainer {
            self.plugin_data::<P>()
        }
        fn plugin_data_mut<P: Plugin>(&mut self) -> &mut P::DataContainer {
            self.plugin_data_mut::<P>()
        }
        fn add_plan(&mut self, time: f64, plan: impl FnOnce(&mut Self) + 'static) {
            self.add_plan(time, plan);
        }
    }

    #[macro_export]
    macro_rules! build_context {
        () => {{ $crate::context::Context::new() }};
    }
}

mod bool_plugin {
    use super::*;

    pub struct BoolPlugin;
    impl Plugin for BoolPlugin {
        type DataContainer = bool;
        fn default() -> Self::DataContainer {
            false
        }
    }
    pub trait BoolExt: PluginContext {
        fn set_bool(&mut self, value: bool) {
            let data_container = self.plugin_data_mut::<BoolPlugin>();
            *data_container = value;
        }
        fn get_bool(&self) -> bool {
            *self.plugin_data::<BoolPlugin>()
        }
    }
    impl<T: PluginContext + ?Sized> BoolExt for T {}
}

mod number_plugin {
    use super::*;

    pub struct NumberPlugin;
    impl Plugin for NumberPlugin {
        type DataContainer = u32;
        fn default() -> Self::DataContainer {
            0
        }
    }

    pub trait NumberExt: PluginContext {
        fn schedule_set_number(&mut self, time: f64, value: u32) {
            self.add_plan(time, move |ctx| {
                ctx.set_number(value);
                println!("Scheduled number set to: {}", value);
            });
        }
        fn set_number(&mut self, value: u32) {
            let data_container = self.plugin_data_mut::<NumberPlugin>();
            *data_container = value;
        }
        fn get_number(&self) -> u32 {
            *self.plugin_data::<NumberPlugin>()
        }
        fn get_bool_as_number(&self) -> u32 {
            if self.get_bool() { 1 } else { 0 }
        }
    }

    pub fn do_stuff_with_numbers(context: &Context) {
        let number = context.get_number();
        println!("Number is: {}", number);
    }

    impl NumberExt for Context {}
}

use bool_plugin::*;
use context::*;
use number_plugin::*;

fn main() {
    let mut context = build_context!();
    context.set_number(42);
    assert_eq!(context.get_number(), 42);
    context.set_bool(true);
    assert_eq!(context.get_bool(), true);
    assert_eq!(context.get_bool_as_number(), 1);
    do_stuff_with_numbers(&context);
    context.add_plan(1.0, |ctx| {
        ctx.set_number(100);
        println!("Plan executed, number set to: {}", ctx.get_number());
    });
    context.schedule_set_number(1.0, 32);
    context.execute();
}
