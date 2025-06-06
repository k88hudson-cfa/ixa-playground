#![allow(unused_variables)]
#![allow(dead_code)]

use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

mod context {
    use super::*;

    pub struct Context {
        pub entity_property_store: HashMap<(TypeId, TypeId), Box<dyn Any>>,
    }
    impl Context {
        pub fn new() -> Self {
            Context {
                entity_property_store: HashMap::new(),
            }
        }
    }
}
pub use context::*;

mod entity {

    use super::*;
    pub trait Entity: 'static {}
    pub struct EntityId<T: Entity> {
        id: usize,
        _marker: std::marker::PhantomData<T>,
    }
    impl<T: Entity> EntityId<T> {
        pub fn new(id: usize) -> Self {
            Self {
                id,
                _marker: std::marker::PhantomData,
            }
        }
        pub fn id(&self) -> usize {
            self.id
        }
    }
    pub trait EntityProperty: 'static {
        type Value: Copy;
        fn default() -> Self::Value;
    }
    pub trait EntityExt {
        fn register_property<E: Entity, P: EntityProperty>(&mut self);
        fn get_property_by_type_id<P: EntityProperty>(
            &self,
            entity_type_id: TypeId,
            id: usize,
        ) -> P::Value;
        fn get_property<E: Entity, P: EntityProperty>(&self, id: EntityId<E>) -> P::Value;
        fn set_property_by_type_id<P: EntityProperty>(
            &mut self,
            entity_type_id: TypeId,
            id: usize,
            value: P::Value,
        );
        fn set_property<E: Entity, P: EntityProperty>(&mut self, id: EntityId<E>, value: P::Value);
    }
    pub struct PropertyVec<P: EntityProperty> {
        values: Vec<P::Value>,
    }
    impl EntityExt for Context {
        fn register_property<E: Entity, P: EntityProperty>(&mut self) {
            let key = (TypeId::of::<E>(), TypeId::of::<P>());
            self.entity_property_store
                .entry(key)
                .or_insert_with(|| Box::new(PropertyVec::<P> { values: vec![] }));
        }
        fn get_property_by_type_id<P: EntityProperty>(
            &self,
            entity_type_id: TypeId,
            id: usize,
        ) -> P::Value {
            let key = (entity_type_id, TypeId::of::<P>());
            self.entity_property_store
                .get(&key)
                .expect("Property not found")
                .downcast_ref::<PropertyVec<P>>()
                .expect("Could not downcast property value")
                .values
                .get(id)
                .map(|v| *v)
                .unwrap_or(P::default())
        }
        fn get_property<E: Entity, P: EntityProperty>(&self, id: EntityId<E>) -> P::Value {
            self.get_property_by_type_id::<P>(TypeId::of::<E>(), id.id())
        }
        fn set_property_by_type_id<P: EntityProperty>(
            &mut self,
            entity_type_id: TypeId,
            id: usize,
            value: P::Value,
        ) {
            let key = (entity_type_id, TypeId::of::<P>());
            self.entity_property_store
                .entry(key)
                .or_insert_with(|| {
                    Box::new(PropertyVec::<P> {
                        values: vec![P::default(); id + 1],
                    })
                })
                .downcast_mut::<PropertyVec<P>>()
                .expect("Could not downcast property value");
        }
        fn set_property<E: Entity, P: EntityProperty>(&mut self, id: EntityId<E>, value: P::Value) {
            self.set_property_by_type_id::<P>(TypeId::of::<E>(), id.id(), value);
        }
    }
}
pub use entity::*;

mod people {
    use super::*;
    pub struct Person;
    impl Entity for Person {}
    pub type PersonId = EntityId<Person>;
}
pub use people::*;

mod settings {
    use super::*;

    pub struct SettingEntity;
    impl Entity for SettingEntity {}

    pub trait SettingType: 'static {}

    pub struct Setting<T: SettingType> {
        _phantom: std::marker::PhantomData<T>,
        pub entity_id: EntityId<SettingEntity>,
    }
    impl<T: SettingType> Setting<T> {
        pub fn new(id: usize) -> Self {
            Self {
                _phantom: std::marker::PhantomData,
                entity_id: EntityId::new(id),
            }
        }
        pub fn get_id(&self) -> usize {
            self.entity_id.id()
        }
    }

    pub trait SettingId {
        fn get_id(&self) -> usize;
    }

    impl<T: SettingType> SettingId for Setting<T> {
        fn get_id(&self) -> usize {
            self.entity_id.id()
        }
    }

    pub struct ItineraryEntry {
        setting: Box<dyn SettingId>,
        ratio: f64,
    }
    impl ItineraryEntry {
        pub fn new<T: SettingType>(setting_id: Setting<T>, ratio: f64) -> Self {
            let setting: Box<dyn SettingId> = Box::new(setting_id);
            Self { setting, ratio }
        }
    }

    pub trait SettingProperty: EntityProperty {}

    pub trait SettingContextExt {
        fn get_setting_property<P: SettingProperty>(&self, setting: Box<dyn SettingId>)
        -> P::Value;
        fn get_settings_members<T: SettingType>(&self, setting_id: Setting<T>) -> Vec<PersonId>;
        fn get_itinerary(&self, person: PersonId) -> Vec<ItineraryEntry>;
        fn get_random_setting(&self, person: PersonId) -> Box<dyn SettingId>;
    }

    pub struct Alpha;
    impl EntityProperty for Alpha {
        type Value = f64;
        fn default() -> Self::Value {
            1.0
        }
    }
    impl SettingProperty for Alpha {}

    pub fn init_settings(context: &mut Context) {
        // This could be lazy or whatever
        context.register_property::<SettingEntity, Alpha>()
    }
}

pub use settings::*;

struct Home;
impl SettingType for Home {}

struct Work;
impl SettingType for Work {}

impl SettingContextExt for Context {
    fn get_setting_property<P: SettingProperty>(&self, setting: Box<dyn SettingId>) -> P::Value {
        let setting_id = setting.get_id();
        let entity_type_id = TypeId::of::<SettingEntity>();
        self.get_property_by_type_id::<P>(entity_type_id, setting_id)
    }
    fn get_settings_members<T: SettingType>(&self, setting_id: Setting<T>) -> Vec<PersonId> {
        vec![]
    }
    fn get_itinerary(&self, person: PersonId) -> Vec<ItineraryEntry> {
        vec![ItineraryEntry::new(Setting::<Home>::new(1), 0.5)]
    }
    fn get_random_setting(&self, person: PersonId) -> Box<dyn SettingId> {
        Box::new(Setting::<Home>::new(1))
    }
}

fn main() {
    let mut context = Context::new();
    init_settings(&mut context);

    context.get_itinerary(PersonId::new(1));
    let setting = context.get_random_setting(PersonId::new(1));
    context.get_setting_property::<Alpha>(setting);
}
