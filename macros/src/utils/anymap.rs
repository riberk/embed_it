use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

#[derive(Debug, Default)]
pub struct AnyMap(HashMap<TypeId, Box<dyn Any>>);

impl AnyMap {
    pub fn get<T: 'static>(&self) -> Option<&T> {
        self.0
            .get(&TypeId::of::<T>())
            .map(|v| v.downcast_ref().unwrap())
    }

    pub fn get_or_default<T: 'static + Default>(&mut self) -> &mut T {
        self.0
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::new(T::default()))
            .downcast_mut()
            .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::AnyMap;

    #[derive(Debug, Default, PartialEq, Eq)]
    struct S1(String);

    #[derive(Debug, Default, PartialEq, Eq)]
    struct S2(String);

    #[derive(Debug, Default, PartialEq, Eq)]
    struct S3(String);

    #[test]
    fn get_or_default_then_get() {
        let mut map = AnyMap::default();
        let s1 = map.get_or_default::<S1>();
        s1.0.push_str("123");
        assert_eq!(map.get_or_default::<S1>().0.as_str(), "123");
        assert_eq!(map.get::<S1>().unwrap().0.as_str(), "123");
    }

    #[test]
    fn different_types() {
        let mut map = AnyMap::default();

        let s = map.get_or_default::<S1>();
        s.0.push('1');

        let s = map.get_or_default::<S2>();
        s.0.push('2');

        let s = map.get_or_default::<S3>();
        s.0.push('3');

        assert_eq!(map.get::<S1>().unwrap().0.as_str(), "1");
        assert_eq!(map.get::<S2>().unwrap().0.as_str(), "2");
        assert_eq!(map.get::<S3>().unwrap().0.as_str(), "3");
    }
}
