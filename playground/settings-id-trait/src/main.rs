#![allow(unused_variables)]
#![allow(dead_code)]

use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

mod ixa {
    use super::*;

    pub struct Context {
        pub data: HashMap<TypeId, Box<dyn Any>>,
    }
    impl Context {
        pub fn new() -> Self {
            Context {
                data: HashMap::new(),
            }
        }
        pub fn add_plan<F>(&mut self, time: f64, plan: F)
        where
            F: FnOnce(&mut Context) + 'static,
        {
            // Here you would schedule the plan to be executed
        }
    }
    pub struct Person;
    #[derive(Copy, Clone)]
    pub struct PersonId {
        pub id: usize,
    }
    impl PersonId {
        pub fn new(id: usize) -> PersonId {
            PersonId { id }
        }
    }
}
pub use ixa::*;

mod settings {
    use super::*;
    pub trait SettingType: 'static {}
    pub struct SettingId<T: SettingType> {
        pub id: usize,
        _phantom: std::marker::PhantomData<T>,
    }
    impl<T: SettingType> SettingId<T> {
        pub fn new(id: usize) -> Self {
            SettingId {
                id,
                _phantom: std::marker::PhantomData::<T>,
            }
        }
        fn id(&self) -> usize {
            self.id
        }
    }

    pub trait AnySettingId {
        fn id(&self) -> usize;
        fn type_id(&self) -> TypeId;
    }

    impl<T: SettingType> AnySettingId for SettingId<T> {
        fn id(&self) -> usize {
            self.id()
        }
        fn type_id(&self) -> TypeId {
            TypeId::of::<SettingId<T>>()
        }
    }

    pub struct ItineraryEntry {
        pub setting: Box<dyn AnySettingId>,
        pub ratio: f64,
    }
    impl ItineraryEntry {
        pub fn new<T: SettingType>(setting_id: SettingId<T>, ratio: f64) -> Self {
            let setting: Box<dyn AnySettingId> = Box::new(setting_id);
            Self { setting, ratio }
        }
    }

    pub trait SettingContextExt {
        fn get_contact(
            &self,
            person_id: PersonId,
            setting_id: &dyn AnySettingId,
        ) -> Option<PersonId>;
        fn get_settings_members<T: SettingType>(&self, setting_id: SettingId<T>) -> Vec<PersonId>;
        fn get_random_setting(&self, person: PersonId) -> Box<dyn AnySettingId>;
        fn get_itinerary(&self, person: PersonId) -> Vec<ItineraryEntry>;
        fn get_alpha(&self, person: PersonId, setting: &dyn AnySettingId) -> f64;
    }

    #[macro_export]
    macro_rules! define_setting_type {
        ($name:ident) => {
            pub struct $name;
            impl SettingType for $name {}
            paste::paste! {
                type [<$name Id>] = SettingId<$name>;
            }
        };
    }
}

pub use settings::*;

define_setting_type!(Home);
define_setting_type!(Work);

impl SettingContextExt for Context {
    fn get_contact(&self, person_id: PersonId, setting_id: &dyn AnySettingId) -> Option<PersonId> {
        Some(PersonId::new(1))
    }
    fn get_alpha(&self, person: PersonId, setting: &dyn AnySettingId) -> f64 {
        let id = setting.id();
        let type_id = setting.type_id();
        // Here you would go look up the alpha by type_id
        0.5
    }
    fn get_settings_members<T: SettingType>(&self, setting_id: SettingId<T>) -> Vec<PersonId> {
        vec![PersonId::new(0)]
    }
    fn get_itinerary(&self, person: PersonId) -> Vec<ItineraryEntry> {
        vec![
            ItineraryEntry::new(HomeId::new(0), 0.5),
            ItineraryEntry::new(WorkId::new(0), 0.5),
        ]
    }
    fn get_random_setting(&self, person: PersonId) -> Box<dyn AnySettingId> {
        Box::new(WorkId::new(0))
    }
}

struct Forecast {
    next_time: f64,
    forecasted_total_infectiousness: f64,
}

fn get_forecast(context: &Context, person: PersonId) -> Option<Forecast> {
    Some(Forecast {
        next_time: 1.0,
        forecasted_total_infectiousness: 0.5,
    })
}
fn evaluate_forecast(
    context: &Context,
    person: PersonId,
    forecasted_total_infectiousness: f64,
) -> bool {
    true
}

pub fn infection_attempt(context: &Context, infector: PersonId) -> Option<PersonId> {
    // Choose a setting
    let setting_id = context.get_random_setting(infector);

    // Get a contact
    let next_contact = context.get_contact(infector, &*setting_id)?;

    // Return early if they're not Susceptible
    // if self.get_person_property(next_contact, InfectionStatus) != InfectionStatus::S {
    //     return None;
    // }

    // Infect them
    // self.infect_person(infector, next_contact, setting_id);

    Some(next_contact)
}

fn schedule_next_forecasted_infection(context: &mut Context, person: PersonId) {
    if let Some(Forecast {
        next_time,
        forecasted_total_infectiousness,
    }) = get_forecast(context, person)
    {
        context.add_plan(next_time, move |context| {
            if evaluate_forecast(context, person, forecasted_total_infectiousness) {
                infection_attempt(context, person);
            }
            schedule_next_forecasted_infection(context, person);
        });
    }
}

fn main() {
    let mut context = Context::new();
    let person = PersonId::new(1);
    schedule_next_forecasted_infection(&mut context, person);
}
