use std::any::{Any, TypeId};
use std::collections::HashMap;

/// Identifies a unique singular instance type. 
pub trait Resource: 'static + Send + Sync {}
impl<T: 'static + Send + Sync> Resource for T {}

/// Collection of globally unique singular data instances (e.g., Time, Input manager).
#[derive(Default)]
pub struct Resources {
    data: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl Resources {
    pub fn new() -> Self {
        Self::default()
    }

    /// Overwrites or inserts a singleton configuration type instance.
    pub fn insert<T: Resource>(&mut self, resource: T) {
        self.data.insert(TypeId::of::<T>(), Box::new(resource));
    }

    /// Fetches an immutable reference to the registered singleton type.
    pub fn get<T: Resource>(&self) -> Option<&T> {
        self.data.get(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_ref::<T>())
    }

    /// Fetches a mutable reference to the registered singleton type.
    pub fn get_mut<T: Resource>(&mut self) -> Option<&mut T> {
        self.data.get_mut(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_mut::<T>())
    }
    
    /// Removes the unique singleton instance from tracking completely, yielding it back to caller.
    pub fn remove<T: Resource>(&mut self) -> Option<T> {
        self.data.remove(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast::<T>().ok().map(|b| *b))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct Score(u32);

    struct NonCopyStruct;

    #[test]
    fn insert_and_get_resources() {
        let mut res = Resources::new();
        res.insert(Score(100));
        
        assert_eq!(res.get::<Score>().unwrap(), &Score(100));
        assert!(res.get::<NonCopyStruct>().is_none());
    }

    #[test]
    fn mutable_resource_access() {
        let mut res = Resources::new();
        res.insert(Score(100));
        
        if let Some(score) = res.get_mut::<Score>() {
            score.0 += 50;
        }
        
        assert_eq!(res.get::<Score>().unwrap().0, 150);
    }

    #[test]
    fn overwrite_and_remove_resources() {
        let mut res = Resources::new();
        res.insert(Score(100));
        
        // Return old value inherently discarded
        res.insert(Score(200));
        assert_eq!(res.get::<Score>().unwrap().0, 200);
        
        let retrieved = res.remove::<Score>().unwrap();
        assert_eq!(retrieved.0, 200);
        assert!(res.get::<Score>().is_none());
    }
}
