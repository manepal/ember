use std::collections::HashMap;

use crate::archetype::{Archetype, ArchetypeId};
use crate::component::{Component, ComponentId};
use crate::entity::{Entity, EntityAllocator};
use crate::query::{Query, WorldQuery};
use crate::resource::{Resource, Resources};

/// The core registry that holds all registered Entities and their data Archetypes.
#[derive(Default)]
pub struct World {
    allocator: EntityAllocator,
    
    /// Map of an active Entity to its current Archetype and row.
    entity_location: HashMap<Entity, EntityLocation>,
    
    /// Storage for all active unique archetype layouts.
    archetypes: HashMap<ArchetypeId, Archetype>,
    next_archetype_id: u32,
    
    /// Maps a sorted combination of Component TypeIds directly to an Archetype ID.
    archetype_index: HashMap<Vec<ComponentId>, ArchetypeId>,
    
    /// Global singular tracking storage map. 
    pub resources: Resources,
}

pub struct EntityLocation {
    pub archetype_id: ArchetypeId,
    pub row: usize,
}

impl World {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Spawns a brand new vacant Entity without any components, and returns an `EntityBuilder`
    /// to fluently attach components to it before yielding it to the World.
    pub fn spawn(&mut self) -> EntityBuilder<'_> {
        let entity = self.allocator.spawn();
        
        // Register the entity in an implicitly "empty" archetype.
        let empty_arch_id = self.get_or_create_archetype(&[]);
        let row = self.archetypes.get_mut(&empty_arch_id).unwrap().allocate(entity);
        
        self.entity_location.insert(entity, EntityLocation {
            archetype_id: empty_arch_id,
            row,
        });

        EntityBuilder {
            world: self,
            entity,
        }
    }

    /// Despawns an entity completely, dropping all of its associated components and freeing internal slots.
    pub fn despawn(&mut self, entity: Entity) -> bool {
        if !self.allocator.is_alive(entity) {
            return false;
        }
        
        // Remove the entity from its archetype (dropping its components)
        if let Some(loc) = self.entity_location.remove(&entity) {
            let arch = self.archetypes.get_mut(&loc.archetype_id).unwrap();
            
            // Pop out the entity by swapping the tail.
            // If the tail entity took its place, update the location lookup pointer.
            let is_last = loc.row == arch.entities.len() - 1;
            
            // Invoke the drop logic for the components.
            for column in &mut arch.columns {
                let sz = column.item_layout.size();
                if sz > 0 {
                    unsafe {
                        let dst_ptr = column.data.add(loc.row * sz);
                        
                        // Drop the old element being deleted.
                        if let Some(drop_fn) = column.item_drop_fn {
                            (drop_fn)(dst_ptr);
                        }
                        
                        // Memory swap the tail 
                        if !is_last {
                            let src_ptr = column.data.add((arch.entities.len() - 1) * sz);
                            std::ptr::copy_nonoverlapping(src_ptr, dst_ptr, sz);
                        }
                    }
                }
            }
            
            let tail_entity = arch.entities.swap_remove(loc.row);
            
            if !is_last {
                // The tail entity shifted slots, update its location map!
                if let Some(tail_loc) = self.entity_location.get_mut(&tail_entity) {
                    tail_loc.row = loc.row;
                }
            }
        }
        
        self.allocator.despawn(entity)
    }

    /// Appends a new component to an existing entity, automatically migrating it to a new archetype.
    pub fn insert_component<T: Component>(&mut self, entity: Entity, component: T) {
        if !self.allocator.is_alive(entity) {
            return;
        }

        let loc = self.entity_location.get(&entity).expect("Entity exists but has no Location.");
        let current_arch_id = loc.archetype_id;
        
        // We evaluate target arch id via a mutable borrow on self
        let target_arch_id = self.find_archetype_with_insert::<T>(current_arch_id);
        
        // Re-borrow the row from location now that `find_archetype...` mutation is done
        let row = self.entity_location.get(&entity).unwrap().row;
        
        if current_arch_id == target_arch_id {
            // Overwriting existing component data inline instead of migrating
            let arch = self.archetypes.get_mut(&current_arch_id).unwrap();
            
            if let Some(&col_idx) = arch.component_indices.get(&ComponentId::of::<T>()) {
                let col = &mut arch.columns[col_idx];
                unsafe {
                    // Drop old
                    let ptr = col.data.add(row * col.item_layout.size());
                    if let Some(drop_fn) = col.item_drop_fn {
                        (drop_fn)(ptr);
                    }
                    col.push(row, component);
                }
            }
            return;
        }

        // We have to extract the two maps cleanly since they borrow `self.archetypes`
        let (mut source_part, mut target_part) = Self::get_archetype_pair_mut(&mut self.archetypes, current_arch_id, target_arch_id);
        let src = source_part.as_mut().unwrap();
        let tgt = target_part.as_mut().unwrap();
        
        unsafe {
            if let Some((shifted_entity, new_row)) = src.migrate_entity_to(entity, tgt) {
                // Update the entity that was swapped down in the source archetype.
                self.entity_location.get_mut(&shifted_entity).unwrap().row = new_row;
            }
            
            // The freshly copied entity is now at the tail of `tgt`
            let new_tgt_row = tgt.len() - 1;
            tgt.insert_component(new_tgt_row, component);
            
            self.entity_location.get_mut(&entity).unwrap().archetype_id = target_arch_id;
            self.entity_location.get_mut(&entity).unwrap().row = new_tgt_row;
        }
    }

    /// Retrieves an immutable reference to the `T` component attached to `entity`.
    pub fn get<T: Component>(&self, entity: Entity) -> Option<&T> {
        let loc = self.entity_location.get(&entity)?;
        let arch = self.archetypes.get(&loc.archetype_id)?;
        arch.get_component::<T>(entity)
    }

    /// Retrieves a mutable reference to the `T` component attached to `entity`.
    pub fn get_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T> {
        let loc = self.entity_location.get(&entity)?;
        let arch = self.archetypes.get_mut(&loc.archetype_id)?;
        arch.get_component_mut::<T>(entity)
    }

    /// Registers a singleton data structure to the entire scope of the World instance.
    pub fn insert_resource<T: Resource>(&mut self, resource: T) {
        self.resources.insert::<T>(resource);
    }

    /// Gets an immutable ref to a singleton globally registered to the World.
    pub fn resource<T: Resource>(&self) -> Option<&T> {
        self.resources.get::<T>()
    }
    
    /// Gets a mutable ref to a singleton globally registered to the World.
    pub fn resource_mut<T: Resource>(&mut self) -> Option<&mut T> {
        self.resources.get_mut::<T>()
    }

    /// Constructs a query for filtering entities matching a specific Component signature `Q`.
    pub fn query<'w, Q: WorldQuery>(&'w self) -> Query<'w, Q> {
        Query::new(self)
    }

    /// Internal accessor for the Query system to iterate Archetypes directly.
    pub(crate) fn archetypes(&self) -> std::collections::hash_map::Values<'_, ArchetypeId, Archetype> {
        self.archetypes.values()
    }
}
// ... continuation of world.rs ...

impl World {
    /// Safe mutable split borrow over the archetype map.
    fn get_archetype_pair_mut(
        archetypes: &mut HashMap<ArchetypeId, Archetype>,
        id1: ArchetypeId,
        id2: ArchetypeId,
    ) -> (Option<&mut Archetype>, Option<&mut Archetype>) {
        if id1 == id2 {
            panic!("Cannot mutably borrow the same archetype twice.");
        }
        
        // We know they are distinct ID keys, so we can split borrow unsafely but package it safely
        unsafe {
            let map_ptr = archetypes as *mut HashMap<ArchetypeId, Archetype>;
            let ptr1 = (*map_ptr).get_mut(&id1).map(|r| r as *mut Archetype);
            let ptr2 = (*map_ptr).get_mut(&id2).map(|r| r as *mut Archetype);

            (
                ptr1.map(|p| &mut *p),
                ptr2.map(|p| &mut *p),
            )
        }
    }
    
    /// Finds or registers exactly which Archetype corresponds to a sorted slice of Component IDs.
    fn get_or_create_archetype(&mut self, components: &[ComponentId]) -> ArchetypeId {
        if let Some(&id) = self.archetype_index.get(components) {
            return id;
        }
        
        // Scaffold definition
        let id = ArchetypeId(self.next_archetype_id);
        self.next_archetype_id += 1;
        
        let arch = Archetype::new(id);
        // Note: we can't call generic `add_column::<T>()` here easily because types are erased inside of `components`.
        // The Archetype graph dynamically constructs types later via `insert_component`, but we'll stub it here 
        // (For a fully featured engine we'd stash fn pointers mapped to type-ids for generic builders).
        
        self.archetypes.insert(id, arch);
        self.archetype_index.insert(components.to_vec(), id);
        
        id
    }

    /// Resolves the destination Archetype definition after appending a new generic type `T`.
    fn find_archetype_with_insert<T: Component>(&mut self, current: ArchetypeId) -> ArchetypeId {
        let arch = self.archetypes.get(&current).unwrap();
        
        let target_comp_id = ComponentId::of::<T>();
        if arch.component_indices.contains_key(&target_comp_id) {
            return current;
        }

        let mut next_components: Vec<ComponentId> = arch.component_indices.keys().copied().collect();
        next_components.push(target_comp_id);
        next_components.sort();
        
        // Let go of `arch` borrow before mutating `self`
        let target_id = self.get_or_create_archetype(&next_components);
        
        // Let's get them both 
        // Because `find_archetype_with_insert` uses a generic, we are 100% guaranteed to know `T` here so 
        // we can dynamically build the columnar backing buffer!
        let (arch_opt, target_opt) = Self::get_archetype_pair_mut(&mut self.archetypes, current, target_id);
        let arch = arch_opt.unwrap();
        let target_arch = target_opt.unwrap();
        
        // Retroactively back-fill all identical columns from `arch` to `target_arch` dynamically via memory offsets if they are missing
        // This acts as a generic-agnostic cloner.
        for &col_idx in arch.component_indices.values() {
            let prev_col = &arch.columns[col_idx];
            if !target_arch.component_indices.contains_key(&prev_col.component_id) {
                target_arch.add_untyped_column(prev_col.clone_empty());
            }
        }
        
        target_arch.add_column::<T>();
        
        target_id
    }
}

pub struct EntityBuilder<'w> {
    world: &'w mut World,
    entity: Entity,
}

impl<'w> EntityBuilder<'w> {
    pub fn insert<T: Component>(self, component: T) -> Self {
        self.world.insert_component(self.entity, component);
        self
    }
    
    pub fn id(&self) -> Entity {
        self.entity
    }
}
// ... continuation of world.rs ...

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, Clone, Copy)]
    struct Position(f32, f32);

    #[derive(Debug, PartialEq, Clone, Copy)]
    struct Velocity(f32, f32);

    #[test]
    fn spawn_and_despawn_entity() {
        let mut world = World::new();
        
        let entity = world.spawn().id();
        assert!(world.allocator.is_alive(entity));
        assert_eq!(world.entity_location.len(), 1);
        
        let success = world.despawn(entity);
        assert!(success);
        assert!(!world.allocator.is_alive(entity));
        assert_eq!(world.entity_location.len(), 0);
    }

    #[test]
    fn test_entity_builder_inserts_components() {
        let mut world = World::new();
        
        let e = world.spawn()
            .insert(Position(10.0, 20.0))
            .insert(Velocity(5.0, -1.0))
            .id();
            
        assert_eq!(world.get::<Position>(e).unwrap().0, 10.0);
        assert_eq!(world.get::<Velocity>(e).unwrap().1, -1.0);
    }
    
    #[test]
    fn archetype_migration_preserves_old_data() {
        let mut world = World::new();
        let e = world.spawn()
            .insert(Position(0.0, 0.0))
            .id();
            
        // At this point e is in an Archetype with purely `Position`
        assert_eq!(world.get::<Position>(e).unwrap().0, 0.0);
        assert!(world.get::<Velocity>(e).is_none());
        
        // Let's migrate it by adding Velocity retroactively
        world.insert_component(e, Velocity(2.5, 3.5));
        
        // Assert old data was copied!
        assert_eq!(world.get::<Position>(e).unwrap().0, 0.0);
        // Assert new data is present!
        assert_eq!(world.get::<Velocity>(e).unwrap().0, 2.5);
    }
    
    #[test]
    fn get_mut_modifies_memory_correctly() {
        let mut world = World::new();
        let e = world.spawn().insert(Position(1.0, 1.0)).id();
        
        if let Some(pos) = world.get_mut::<Position>(e) {
            pos.0 = 5.0;
        }
        
        assert_eq!(world.get::<Position>(e).unwrap().0, 5.0);
    }
}
