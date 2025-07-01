use std::{
    any::{Any, TypeId},
    cell::UnsafeCell,
    collections::HashMap,
    collections::hash_map::Entry,
};

pub trait Key: 'static {
    type Value;
}

pub struct TypeContainer {
    container: UnsafeCell<InnerContainer>,
}

impl TypeContainer {
    pub fn new() -> TypeContainer {
        TypeContainer {
            container: UnsafeCell::new(InnerContainer::new()),
        }
    }

    pub fn try_insert<K: Key>(&self, value: K::Value) -> Result<(), String> {
        unsafe { (*self.container.get()).try_insert::<K>(value) }
    }

    pub fn get<K: Key>(&self) -> Option<&K::Value> {
        unsafe { (*self.container.get()).get::<K>() }
    }

    pub fn get_mut<K: Key>(&mut self) -> Option<&mut K::Value> {
        unsafe { (*self.container.get()).get_mut::<K>() }
    }
}

struct InnerContainer {
    map: HashMap<TypeId, Box<dyn Any>>,
}

impl InnerContainer {
    fn new() -> InnerContainer {
        InnerContainer {
            map: HashMap::new(),
        }
    }

    fn get<K: Key>(&self) -> Option<&K::Value> {
        self.map
            .get(&TypeId::of::<K>())
            .and_then(|boxed| boxed.downcast_ref())
    }

    fn get_mut<K: Key>(&mut self) -> Option<&mut K::Value> {
        self.map
            .get_mut(&TypeId::of::<K>())
            .and_then(|boxed| boxed.downcast_mut())
    }

    fn try_insert<K: Key>(&mut self, value: K::Value) -> Result<(), String> {
        match self.map.entry(TypeId::of::<K>()) {
            Entry::Occupied(_) => return Err("Container already contains key".into()),
            Entry::Vacant(vacant_entry) => {
                vacant_entry.insert(Box::new(value));
                return Ok(());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct A;
    impl Key for A {
        type Value = usize;
    }

    struct B;
    impl Key for B {
        type Value = bool;
    }

    struct C;
    impl Key for C {
        type Value = ();
    }

    struct D;
    impl Key for D {
        type Value = f64;
    }

    #[test]
    fn test_insert() {
        let container = TypeContainer::new();

        assert!(container.get::<A>().is_none());
        assert!(container.get::<B>().is_none());
        assert!(container.get::<C>().is_none());
        assert!(container.get::<D>().is_none());

        assert!(container.try_insert::<A>(1).is_ok());
        assert!(container.get::<A>().is_some_and(|x| *x == 1));
        assert!(container.get::<B>().is_none());
        assert!(container.get::<C>().is_none());
        assert!(container.get::<D>().is_none());

        assert!(container.try_insert::<B>(true).is_ok());
        assert!(container.get::<A>().is_some_and(|x| *x == 1));
        assert!(container.get::<B>().is_some_and(|x| *x));
        assert!(container.get::<C>().is_none());
        assert!(container.get::<D>().is_none());

        assert!(container.try_insert::<C>(()).is_ok());
        assert!(container.get::<A>().is_some_and(|x| *x == 1));
        assert!(container.get::<B>().is_some_and(|x| *x));
        assert!(container.get::<C>().is_some_and(|x| x.eq(&())));
        assert!(container.get::<D>().is_none());

        assert!(container.try_insert::<D>(1.0).is_ok());
        assert!(container.get::<A>().is_some_and(|x| *x == 1));
        assert!(container.get::<B>().is_some_and(|x| *x));
        assert!(container.get::<C>().is_some_and(|x| x.eq(&())));
        assert!(container.get::<D>().is_some_and(|x| x.eq(&1.0)));
    }

    #[test]
    fn test_mutate() {
        let mut container = TypeContainer::new();

        let _ = container.try_insert::<A>(1);
        let _ = container.try_insert::<B>(true);
        let _ = container.try_insert::<C>(());
        let _ = container.try_insert::<D>(1.0);

        let a = container.get_mut::<A>().unwrap();
        *a = 2;
        assert!(container.get::<A>().is_some_and(|x| *x == 2));
        assert!(container.get::<B>().is_some_and(|x| *x));
        assert!(container.get::<C>().is_some_and(|x| x.eq(&())));
        assert!(container.get::<D>().is_some_and(|x| x.eq(&1.0)));

        let b = container.get_mut::<B>().unwrap();
        *b = false;
        assert!(container.get::<A>().is_some_and(|x| *x == 2));
        assert!(container.get::<B>().is_some_and(|x| !*x));
        assert!(container.get::<C>().is_some_and(|x| x.eq(&())));
        assert!(container.get::<D>().is_some_and(|x| x.eq(&1.0)));

        let c = container.get_mut::<C>().unwrap();
        *c = ();
        assert!(container.get::<A>().is_some_and(|x| *x == 2));
        assert!(container.get::<B>().is_some_and(|x| !*x));
        assert!(container.get::<C>().is_some_and(|x| x.eq(&())));
        assert!(container.get::<D>().is_some_and(|x| x.eq(&1.0)));

        let d = container.get_mut::<D>().unwrap();
        *d = 2.0;
        assert!(container.get::<A>().is_some_and(|x| *x == 2));
        assert!(container.get::<B>().is_some_and(|x| !*x));
        assert!(container.get::<C>().is_some_and(|x| x.eq(&())));
        assert!(container.get::<D>().is_some_and(|x| x.eq(&2.0)));
    }

    #[test]
    fn test_reference_insert() {
        let container = TypeContainer::new();

        // Hold on to shared reference while inserting
        let _ = container.try_insert::<A>(1);
        let a = container.get::<A>().unwrap();

        // Force internal array to be resized
        let _ = container.try_insert::<B>(true);
        let _ = container.try_insert::<C>(());
        let _ = container.try_insert::<D>(1.0);
        assert!(container.get::<B>().is_some_and(|x| *x));
        assert!(container.get::<C>().is_some_and(|x| x.eq(&())));
        assert!(container.get::<D>().is_some_and(|x| x.eq(&1.0)));

        // Check heap reference is still valid
        assert_eq!(a, &1);
    }

    #[test]
    fn test_double_insert() {
        let container = TypeContainer::new();
        assert!(container.try_insert::<A>(1).is_ok());
        assert!(container.try_insert::<A>(2).is_err());
        assert!(container.get::<A>().is_some_and(|x| *x == 1));
    }
}
