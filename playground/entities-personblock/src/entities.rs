use std::{
    any::{Any, TypeId},
    cell::{OnceCell, RefCell},
    collections::HashMap,
    marker::PhantomData,
    sync::{LazyLock, Mutex},
};

use crate::{
    context::{Context, PluginContext},
    define_data_plugin,
    vec_cell::VecCell,
};

static ENTITY_PROPERTIES: LazyLock<
    Mutex<RefCell<HashMap<(TypeId, TypeId), Box<fn() -> Box<dyn Any>>>>>,
> = LazyLock::new(|| Mutex::new(RefCell::new(HashMap::new())));

fn get_property_once_cell<P: Property>() -> Box<dyn Any> {
    Box::new(OnceCell::<VecCell<Option<P::Value>>>::new())
}

pub fn add_property_to_registry<E: Entity, P: PropertyOf<E>>() {
    ENTITY_PROPERTIES.lock().unwrap().borrow_mut().insert(
        (TypeId::of::<E>(), TypeId::of::<P>()),
        Box::new(get_property_once_cell::<P>),
    );
}

pub fn get_property_data() -> HashMap<(TypeId, TypeId), Box<dyn Any>> {
    ENTITY_PROPERTIES
        .lock()
        .unwrap()
        .borrow()
        .iter()
        .map(|(key, function)| (*key, function()))
        .collect()
}

pub trait Entity: 'static {}

pub struct EntityId<T> {
    id: usize,
    _phantom: PhantomData<T>,
}

impl<T: Entity> EntityId<T> {
    fn new(id: usize) -> EntityId<T> {
        EntityId {
            id,
            _phantom: PhantomData,
        }
    }
}
impl<T: Entity> Clone for EntityId<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            _phantom: self._phantom.clone(),
        }
    }
}
impl<T: Entity> Copy for EntityId<T> {}

pub trait Property: 'static {
    type Value: Copy;
}

pub trait PropertyOf<E: Entity>: Property {
    fn compute(context: &impl PluginContext, entity: EntityId<E>) -> Self::Value;
}

struct EntityData {
    // Will store for (Entity, Property) TypeId pairs the Vec of property values
    property_data: HashMap<(TypeId, TypeId), Box<dyn Any>>,
    // (Maybe) Will store for each Entity a Vec of initializers
    // initializers: HashMap<TypeId, Box<dyn Any>>,
    entity_count: HashMap<TypeId, usize>,
}

impl EntityData {
    pub fn new() -> EntityData {
        Self {
            property_data: get_property_data(),
            entity_count: HashMap::new(),
        }
    }
}

define_data_plugin!(EntityPlugin, EntityData, |_context| EntityData::new());

pub trait EntityExt: PluginContext {
    fn add_entity<E: Entity>(&mut self, _entity: E) -> EntityId<E> {
        let data = self.get_data_mut(EntityPlugin);
        let count = data.entity_count.entry(TypeId::of::<E>()).or_default();
        let id = EntityId::new(*count);
        *count += 1;
        id
    }

    fn get_property<E: Entity, P: PropertyOf<E>>(&self, id: EntityId<E>, _property: P) -> P::Value
    where
        Self: Sized + 'static,
    {
        let data = self.get_data(EntityPlugin);
        let property_vec_cell = data
            .property_data
            .get(&(TypeId::of::<E>(), TypeId::of::<P>()))
            .expect("Property not defined")
            .downcast_ref::<OnceCell<VecCell<Option<P::Value>>>>()
            .unwrap()
            .get_or_init(|| VecCell::new());
        if let Some(value) = property_vec_cell.get_or_extend(id.id, None) {
            return value;
        } else {
            let value = P::compute(self, id);
            property_vec_cell.set(id.id, Some(value));
            return value;
        }
    }

    fn set_property<E: Entity, P: PropertyOf<E>>(
        &mut self,
        id: EntityId<E>,
        _property: P,
        value: P::Value,
    ) {
        let data = self.get_data(EntityPlugin);
        data.property_data
            .get(&(TypeId::of::<E>(), TypeId::of::<P>()))
            .expect("Property not defined")
            .downcast_ref::<OnceCell<VecCell<Option<P::Value>>>>()
            .unwrap()
            .get_or_init(|| VecCell::new())
            .set_or_extend(id.id, Some(value), None);
    }
}

impl EntityExt for Context {}

#[macro_export]
macro_rules! define_entity {
    ($entity:ident) => {
        struct $entity;

        impl $crate::entities::Entity for $entity {}
    };
}

#[macro_export]
macro_rules! define_entity_property {
    ($entity:ident, $property:ident, $value:ty, |$ctx:ident, $id:ident| $body:expr) => {
        struct $property;

        impl Property for $property {
            type Value = $value;
        }

        impl PropertyOf<$entity> for $property {
            fn compute($ctx: &impl PluginContext, id: EntityId<$entity>) -> $value {
                $body
            }
        }

        paste::paste! {
            #[ctor::ctor]
            fn [<_register_property_$entity:snake _$property:snake>]() {
                $crate::entities::add_property_to_registry::<$entity, $property>()
            }
        }
    };
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::context::Context;

    define_entity!(Person);
    define_entity_property!(Person, Age, u8, |_context, _id| 10);

    #[test]
    fn test() {
        let mut context = Context::new();

        let person = context.add_entity(Person);
        assert_eq!(context.get_property(person, Age), 10);
        let other_person = context.add_entity(Person);
        context.set_property(other_person, Age, 20);
        assert_eq!(context.get_property(other_person, Age), 20);
    }
}
