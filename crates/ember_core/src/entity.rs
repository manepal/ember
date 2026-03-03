//! Entity definition and generation ID index allocator.

/// A unique identifier for an ECS entity.
/// 
/// Internally this relies on a Generational Index:
/// - `id` is the index into a component array.
/// - `generation` increments every time the `id` is recycled, detecting stale references.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Entity {
    id: u32,
    generation: u32,
}

impl Entity {
    pub fn new(id: u32, generation: u32) -> Self {
        Self { id, generation }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn generation(&self) -> u32 {
        self.generation
    }
}

/// Dispenses Entity IDs, recycles freed IDs, and increments generations to prevent the ABA problem.
#[derive(Debug, Default)]
pub struct EntityAllocator {
    /// Maps a given `id` (index) to its current `generation`
    generations: Vec<u32>,
    /// Unused, freed IDs waiting to be recycled.
    free_indices: Vec<u32>,
    /// The next contiguous unused index.
    next_index: u32,
}

impl EntityAllocator {
    pub fn new() -> Self {
        Self::default()
    }

    /// Allocates the next available optimal Entity ID.
    pub fn spawn(&mut self) -> Entity {
        if let Some(id) = self.free_indices.pop() {
            // Re-use an old slot, the generation was incremented when it despawned.
            Entity::new(id, self.generations[id as usize])
        } else {
            // Acquire a brand new slot
            let id = self.next_index;
            self.next_index += 1;
            
            // Initial generation is 0
            self.generations.push(0);
            
            Entity::new(id, 0)
        }
    }

    /// Despawns an entity, recycling its ID and incrementing the storage generation map.
    /// Returns `true` if the entity was valid and despawned, `false` if the entity was already stale/dead.
    pub fn despawn(&mut self, entity: Entity) -> bool {
        if self.is_alive(entity) {
            // Increment the generation, permanently invalidating any stored usages of the old `entity` reference.
            let idx = entity.id as usize;
            self.generations[idx] += 1;
            
            // Mark the slot free
            self.free_indices.push(entity.id);
            true
        } else {
            false
        }
    }

    /// Checks if a provided `Entity` handle matches the current active generation of its internal `id`.
    /// E.g ensures the pointer hasn't become stale.
    pub fn is_alive(&self, entity: Entity) -> bool {
        if let Some(&current_generation) = self.generations.get(entity.id as usize) {
            entity.generation == current_generation
        } else {
            // The ID hasn't been allocated yet.
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawn_initializes_with_zero_generation() {
        let mut allocator = EntityAllocator::new();
        let e = allocator.spawn();
        
        assert_eq!(e.id(), 0);
        assert_eq!(e.generation(), 0);
        assert!(allocator.is_alive(e));
    }

    #[test]
    fn spawn_increments_next_index() {
        let mut allocator = EntityAllocator::new();
        let e1 = allocator.spawn();
        let e2 = allocator.spawn();
        
        assert_eq!(e1.id(), 0);
        assert_eq!(e2.id(), 1);
        
        assert_eq!(e1.generation(), 0);
        assert_eq!(e2.generation(), 0);
    }

    #[test]
    fn despawning_increments_generation_and_invalidates_original_entity() {
        let mut allocator = EntityAllocator::new();
        let e = allocator.spawn();
        
        assert!(allocator.is_alive(e));
        
        let despawned = allocator.despawn(e);
        assert!(despawned);
        
        // The original handle is now stale!
        assert!(!allocator.is_alive(e));
    }

    #[test]
    fn despawning_is_idempotent() {
        let mut allocator = EntityAllocator::new();
        let e = allocator.spawn();
        
        assert!(allocator.despawn(e));
        assert!(!allocator.despawn(e)); // Double despawn should fail gracefully
    }

    #[test]
    fn despawned_ids_are_recycled_with_new_generations() {
        let mut allocator = EntityAllocator::new();
        let e1 = allocator.spawn();
        assert_eq!(e1.id(), 0);
        
        allocator.despawn(e1);
        
        // The freelist should pop '0' back off, yielding generation 1.
        let e2 = allocator.spawn();
        assert_eq!(e2.id(), 0);
        assert_eq!(e2.generation(), 1);
        
        // Ensure e1 is still considered dead and e2 is alive
        assert!(!allocator.is_alive(e1));
        assert!(allocator.is_alive(e2));
    }

    #[test]
    fn spawn_exhausts_free_list_before_increasing_limit() {
        let mut allocator = EntityAllocator::new();
        let e1 = allocator.spawn();
        let _e2 = allocator.spawn();
        let e3 = allocator.spawn(); // IDs 0, 1, 2
        
        allocator.despawn(e1);
        allocator.despawn(e3);
        
        // Free list now has [0, 2] (order depends on pop implementation but usually LIFO so [2, 0])
        let e4 = allocator.spawn();
        let e5 = allocator.spawn();
        
        // We should reuse IDs from the free array before expanding.
        assert!(e4.id() == 0 || e4.id() == 2);
        assert!(e5.id() == 0 || e5.id() == 2);
        assert_ne!(e4.id(), e5.id());
        
        // Both reused elements should have generation 1.
        assert_eq!(e4.generation(), 1);
        assert_eq!(e5.generation(), 1);
        
        // Next allocation stretches array (id 3).
        let e6 = allocator.spawn();
        assert_eq!(e6.id(), 3);
        assert_eq!(e6.generation(), 0);
    }
}
