use crate::archetype::Archetype;
use crate::component::{Component, ComponentId};
use crate::system::Access;
use crate::world::World;

/// Trait implemented by types that can be fetched from the World via Queries.
/// This acts as the fundamental building block for typed iteration over Archetypes.
pub trait WorldQuery {
    type Item<'w>;
    
    /// Collects the ComponentIds required by this query to match an Archetype.
    fn required_components() -> Vec<ComponentId>;
    
    /// Collects the memory access rights (read vs write) this query enforces.
    fn access() -> Access;
}

/// A Query allows filtering the World's entities and iterating over those that match
/// a specific set of components Q. 
pub struct Query<'w, Q: WorldQuery> {
    world: &'w World,
    _marker: std::marker::PhantomData<Q>,
}

impl<'w, Q: WorldQuery> Query<'w, Q> {
    pub fn new(world: &'w World) -> Self {
        Self {
            world,
            _marker: std::marker::PhantomData,
        }
    }

    /// Fetches an Iterator dynamically querying all Archetypes for matching items.
    pub fn iter(&self) -> QueryIter<'_, Q> {
        let reqs = Q::required_components();
        let mut matched_archetypes = Vec::new();

        for arch in self.world.archetypes() {
            let matches = reqs.iter().all(|c| arch.component_indices.contains_key(c));
            if matches && !arch.entities.is_empty() {
                matched_archetypes.push(arch);
            }
        }

        QueryIter {
            archetypes: matched_archetypes,
            current_arch_idx: 0,
            current_row: 0,
            _marker: std::marker::PhantomData,
        }
    }
}

// Immutable Component Queries
impl<T: Component> WorldQuery for &T {
    type Item<'w> = &'w T;

    fn required_components() -> Vec<ComponentId> {
        vec![ComponentId::of::<T>()]
    }
    
    fn access() -> Access {
        let mut acc = Access::default();
        acc.reads.push(std::any::TypeId::of::<T>());
        acc
    }
}

// Mutable Component Queries
impl<T: Component> WorldQuery for &mut T {
    type Item<'w> = &'w mut T;

    fn required_components() -> Vec<ComponentId> {
        vec![ComponentId::of::<T>()]
    }
    
    fn access() -> Access {
        let mut acc = Access::default();
        acc.writes.push(std::any::TypeId::of::<T>());
        acc
    }
}

// Macro to generate implementations for Tuples (e.g. `Query<(&Position, &Velocity)>`)
macro_rules! impl_world_query_tuple {
    ($($name:ident),*) => {
        impl<$($name: WorldQuery),*> WorldQuery for ($($name,)*) {
            type Item<'w> = ($($name::Item<'w>,)*);

            fn required_components() -> Vec<ComponentId> {
                let mut reqs = Vec::new();
                $(
                    reqs.extend($name::required_components());
                )*
                reqs
            }
            
            fn access() -> Access {
                let mut acc = Access::default();
                $( acc.merge(&$name::access()); )*
                acc
            }
        }
        
        impl<'w, $($name: Fetch<'w>),*> Fetch<'w> for ($($name,)*) {
            type Item = ($($name::Item,)*);

            unsafe fn fetch(archetype: &'w Archetype, row: usize) -> Self::Item {
                ($($name::fetch(archetype, row),)*)
            }
        }
    };
}

impl_world_query_tuple!(A);
impl_world_query_tuple!(A, B);
impl_world_query_tuple!(A, B, C);
impl_world_query_tuple!(A, B, C, D);
impl_world_query_tuple!(A, B, C, D, E);
impl_world_query_tuple!(A, B, C, D, E, F);
// ... continuation of query.rs ...

// Helper trait to allow types to extract their data from a specific Archetype row
pub trait Fetch<'w> {
    type Item;

    /// # Safety
    /// The caller must guarantee that the `Archetype` has the required components.
    unsafe fn fetch(archetype: &'w Archetype, row: usize) -> Self::Item;
}

// Implement Fetch for immutable borrows
impl<'w, T: Component> Fetch<'w> for &T {
    type Item = &'w T;

    unsafe fn fetch(archetype: &'w Archetype, row: usize) -> Self::Item {
        let col_idx = *archetype.component_indices.get(&ComponentId::of::<T>()).unwrap();
        archetype.columns[col_idx].get_unchecked::<T>(row)
    }
}

// Implement Fetch for mutable borrows
impl<'w, T: Component> Fetch<'w> for &mut T {
    type Item = &'w mut T;

    unsafe fn fetch(archetype: &'w Archetype, row: usize) -> Self::Item {
        // We cast away the immutable lifetime from the outer World borrow here
        // The query system will dynamically check Aliasing rules to prevent split `&mut` bugs!
        // Unsafe block here is an explicit bridge from ECS type erasure to typed output.
        let arch_ptr = archetype as *const Archetype as *mut Archetype;
        let col_idx = *(&*arch_ptr).component_indices.get(&ComponentId::of::<T>()).unwrap();
        (&mut (*arch_ptr).columns)[col_idx].get_mut_unchecked::<T>(row)
    }
}

// We map `WorldQuery` tuple items to `Fetch` extractors
pub struct QueryIter<'w, Q: WorldQuery> {
    archetypes: Vec<&'w Archetype>,
    current_arch_idx: usize,
    current_row: usize,
    _marker: std::marker::PhantomData<Q>,
}

impl<'w, Q: WorldQuery> Iterator for QueryIter<'w, Q> 
where 
    Q: Fetch<'w, Item = <Q as WorldQuery>::Item<'w>> 
{
    type Item = <Q as WorldQuery>::Item<'w>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.current_arch_idx >= self.archetypes.len() {
                return None;
            }

            let arch = self.archetypes[self.current_arch_idx];
            let row = self.current_row;
            
            if row < arch.entities.len() {
                self.current_row += 1;
                // Safety: `self.archetypes` is pre-filtered to only contain archetypes with `Q` components
                return Some(unsafe { Q::fetch(arch, row) });
            } else {
                // Move to the next archetype
                self.current_arch_idx += 1;
                self.current_row = 0;
            }
        }
    }
}
// ... continuation of query.rs ...

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct Position(f32, f32);

    #[derive(Debug, PartialEq)]
    struct Velocity(f32, f32);

    #[derive(Debug, PartialEq)]
    struct Name(&'static str);

    #[test]
    fn query_single_component() {
        let mut world = World::new();
        
        world.spawn().insert(Position(0.0, 0.0)).insert(Velocity(1.0, 0.0)).id();
        world.spawn().insert(Position(1.0, 1.0)).id();
        world.spawn().insert(Velocity(0.0, 1.0)).id();

        let query = world.query::<&Position>();
        let positions: Vec<_> = query.iter().collect();

        assert_eq!(positions.len(), 2);
        assert!(positions.contains(&&Position(0.0, 0.0)));
        assert!(positions.contains(&&Position(1.0, 1.0)));
    }

    #[test]
    fn query_tuple_components() {
        let mut world = World::new();
        
        let e1 = world.spawn().insert(Position(0.0, 0.0)).insert(Velocity(1.0, 2.0)).insert(Name("Player")).id();
        world.spawn().insert(Position(1.0, 1.0)).insert(Name("Tree")).id();
        let e3 = world.spawn().insert(Position(10.0, 10.0)).insert(Velocity(-1.0, -1.0)).id();

        let query = world.query::<(&mut Position, &Velocity)>();
        
        let mut count = 0;
        for (pos, vel) in query.iter() {
            pos.0 += vel.0;
            pos.1 += vel.1;
            count += 1;
        }
        
        assert_eq!(count, 2);

        // Verify mutations applied successfully to the correct entities
        assert_eq!(world.get::<Position>(e1).unwrap(), &Position(1.0, 2.0));
        assert_eq!(world.get::<Position>(e3).unwrap(), &Position(9.0, 9.0));
    }
}
