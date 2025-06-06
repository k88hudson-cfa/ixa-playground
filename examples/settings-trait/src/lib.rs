fn main() {
    println!("Hello, world!");
}

mod context {
    pub struct Context {}
    impl Context {
        pub fn new() -> Self {
            Context {}
        }
    }
}
pub use context::*;

mod entity {
    pub trait Entity {}
    pub struct EntityId<T: Entity> {
        id: u64,
        _marker: std::marker::PhantomData<T>,
    }
    impl<T: Entity> EntityId<T> {
        pub fn new(id: u64) -> Self {
            Self {
                id,
                _marker: std::marker::PhantomData,
            }
        }
        pub fn id(&self) -> u64 {
            self.id
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
    pub struct SettingType<T: 'static> {
        _marker: std::marker::PhantomData<T>,
    }
    pub type SettingId<T> = EntityId<SettingType<T>>;

    pub trait Setting {
        fn get_members(&self) -> Vec<PersonId>;
    }

    pub struct ItineraryEntry {
        setting: Box<dyn Setting>,
        ratio: f64,
    }
    impl ItineraryEntry {
        pub fn new<T>(setting_id: SettingId<T>, ratio: f64) -> Self
        where
            SettingType<T>: Entity,
            SettingId<T>: Setting,
        {
            let setting: Box<dyn Setting> = Box::new(setting_id);
            Self { setting, ratio }
        }
    }

    pub trait SettingContextExt {
        fn get_settings_members<T>(&self, setting_id: SettingId<T>) -> Vec<PersonId>
        where
            SettingType<T>: Entity;
        fn get_itinerary(&self, person: PersonId) -> Vec<ItineraryEntry>;
        fn get_random_setting(&self, person: PersonId) -> Box<dyn Setting>;
    }
}

pub use settings::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_people() {
        let person_id = PersonId::new(1);
        assert_eq!(person_id.id(), 1);
    }

    struct Home;
    impl Entity for SettingType<Home> {}
    type HomeId = EntityId<SettingType<Home>>;
    impl Setting for SettingId<Home> {
        fn get_members(&self) -> Vec<PersonId> {
            vec![PersonId::new(1), PersonId::new(2)]
        }
    }

    struct Work;
    impl Entity for SettingType<Work> {}
    type WorkId = EntityId<SettingType<Work>>;
    impl Setting for SettingId<Work> {
        fn get_members(&self) -> Vec<PersonId> {
            vec![PersonId::new(1), PersonId::new(2)]
        }
    }

    impl SettingContextExt for Context {
        fn get_settings_members<T>(&self, setting_id: SettingId<T>) -> Vec<PersonId>
        where
            SettingType<T>: Entity,
        {
            vec![PersonId::new(1), PersonId::new(2)]
        }
        fn get_itinerary(&self, person: PersonId) -> Vec<ItineraryEntry> {
            vec![
                ItineraryEntry::new(HomeId::new(1), 0.5),
                ItineraryEntry::new(WorkId::new(2), 0.5),
            ]
        }
        fn get_random_setting(&self, person: PersonId) -> Box<dyn Setting> {
            Box::new(SettingId::<Home>::new(1))
        }
    }

    #[test]
    fn test_settings() {
        let context = Context::new();
        context.get_itinerary(PersonId::new(1));
        context.get_random_setting(PersonId::new(1));
    }
}
