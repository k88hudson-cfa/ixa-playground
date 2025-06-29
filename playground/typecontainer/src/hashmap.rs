use crate::Key;
use std::{
    any::{Any, TypeId},
    cell::UnsafeCell,
    collections::HashMap,
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
        let type_id = TypeId::of::<K>();
        if self.map.contains_key(&type_id) {
            return Err("Container already contains key".into());
        }
        let boxed_value = Box::new(value);
        self.map.insert(type_id, boxed_value);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_suite;
    test_suite!(TypeContainer);
}
