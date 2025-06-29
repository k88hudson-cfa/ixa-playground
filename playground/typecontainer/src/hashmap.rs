use crate::Key;
use std::{
    any::{Any, TypeId},
    cell::UnsafeCell,
    collections::HashMap,
    collections::hash_map::Entry,
};

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
    use crate::test_suite;
    test_suite!(TypeContainer);
}
