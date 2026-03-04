use std::{
    alloc::{alloc, dealloc, realloc, Layout},
    collections::HashMap,
    ptr,
};

use crate::component::{Component, ComponentId};
use crate::entity::Entity;

/// Identifies uniquely distinct combination of types an entity possesses.
/// Serves as the key/edge identifier between `Archetype` instances.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ArchetypeId(pub u32);

/// Stores contiguous dense columns of components for homogeneous Entities.
/// All entities in an Archetype have the exact same combination of `Component`s.
pub struct Archetype {
    pub id: ArchetypeId,
    /// Maps a `ComponentId` to its linear column index.
    pub component_indices: HashMap<ComponentId, usize>,
    /// Array of type-erased untyped column arrays.
    pub columns: Vec<Column>,
    /// The entities present in this Archetype. Used to backtrack dense slots back to ECS Handles.
    pub entities: Vec<Entity>,
}

/// An untyped, homogeneous sequence of raw bytes representing contiguous components.
pub struct Column {
    pub component_id: ComponentId,
    pub item_layout: Layout,
    pub item_drop_fn: Option<unsafe fn(*mut u8)>,
    // Start of the buffer
    pub data: *mut u8,
    // Size capacity
    pub capacity: usize,
}

impl Drop for Column {
    fn drop(&mut self) {
        if self.data.is_null() {
            return;
        }

        // Determine aggregate layout dynamically to dealloc the whole array
        if self.item_layout.size() > 0 && self.capacity > 0 {
            unsafe {
                let array_layout = Layout::from_size_align_unchecked(
                    self.item_layout.size() * self.capacity,
                    self.item_layout.align(),
                );
                dealloc(self.data, array_layout);
            }
        }
    }
}

unsafe impl Send for Column {}
unsafe impl Sync for Column {}

impl Column {
    pub fn new<T: Component>(capacity: usize) -> Self {
        let layout = Layout::new::<T>();
        let data = if layout.size() == 0 || capacity == 0 {
            // Dangling pointer aligned to `T`
            ptr::NonNull::<T>::dangling().as_ptr() as *mut u8
        } else {
            let buf_layout = Layout::from_size_align(layout.size() * capacity, layout.align())
                .expect("Failed to create Column Layout alignment.");
            unsafe { alloc(buf_layout) }
        };

        // Cache the safe drop function pointer for T if necessary.
        let drop_fn = if std::mem::needs_drop::<T>() {
            Some(Self::drop_ptr::<T> as unsafe fn(*mut u8))
        } else {
            None
        };

        Self {
            component_id: ComponentId::of::<T>(),
            item_layout: layout,
            item_drop_fn: drop_fn,
            data,
            capacity,
        }
    }

    /// Clones the layout definition to spawn an empty identical typed-buffer.
    pub fn clone_empty(&self) -> Self {
        let data = if self.item_layout.size() == 0 {
            ptr::NonNull::new(self.item_layout.align() as *mut u8)
                .unwrap()
                .as_ptr()
        } else {
            // For capacity 0, we just need a properly aligned dangling ptr
            // Using `align` value as the address works for `NonNull::dangling()` internals conventionally,
            // but std::ptr::NonNull::new(align as *mut u8) is the standard method for erasing types while preserving alignment cleanly.
            ptr::NonNull::new(self.item_layout.align() as *mut u8)
                .unwrap()
                .as_ptr()
        };

        Self {
            component_id: self.component_id,
            item_layout: self.item_layout,
            item_drop_fn: self.item_drop_fn,
            data,
            capacity: 0,
        }
    }

    /// Extends the dynamically sized raw buffer to seat a new upper-limit of instances.
    ///
    /// # Safety
    /// Calling this assumes that the capacity passed is actually strictly bounds checked elsewhere.
    /// It performs raw heap allocations for pointer manipulation directly.
    pub unsafe fn grow(&mut self, new_capacity: usize) {
        if self.item_layout.size() == 0 || new_capacity <= self.capacity {
            return;
        }

        let old_layout = Layout::from_size_align_unchecked(
            self.item_layout.size() * self.capacity,
            self.item_layout.align(),
        );

        let new_size = self.item_layout.size() * new_capacity;

        let new_data = if self.capacity == 0 {
            alloc(Layout::from_size_align_unchecked(
                new_size,
                self.item_layout.align(),
            ))
        } else {
            realloc(self.data, old_layout, new_size)
        };

        if new_data.is_null() {
            std::alloc::handle_alloc_error(Layout::from_size_align_unchecked(
                new_size,
                self.item_layout.align(),
            ));
        }

        self.data = new_data;
        self.capacity = new_capacity;
    }

    /// Moves a raw component structure into the contiguous column buffer, claiming ownership.
    ///
    /// # Safety
    /// The caller must guarantee that `index` is a valid allocated row within the `capacity`.
    #[inline]
    pub unsafe fn push<T: Component>(&mut self, index: usize, component: T) {
        // For zero-sized types, we don't actually write memory.
        let sz = self.item_layout.size();
        if sz == 0 {
            // "Drop" the original item essentially, as ZST references do not occupy space.
            std::mem::forget(component);
            return;
        }

        // Target slot.
        let dst = self.data.add(index * sz) as *mut T;
        ptr::write(dst, component);
    }

    /// Reads the internal `index` element in the buffer via an unsafe ptr coercion.
    ///
    /// # Safety
    /// Caller must guarantee the index is bounds checked to the len of the backing architecture slice.
    #[inline]
    pub unsafe fn get_ptr(&self, index: usize) -> *const u8 {
        self.data.add(index * self.item_layout.size())
    }

    /// Translates the internal `index` element in the buffer back into type `T`.
    ///
    /// # Safety
    /// Caller must guarantee `index` points to a valid sequence of bytes of layout `T`.
    #[inline]
    pub unsafe fn get_unchecked<T: Component>(&self, index: usize) -> &T {
        &*(self.get_ptr(index) as *const T)
    }

    /// Mutably translates the internal `index` element in the buffer back into type `T`.
    ///
    /// # Safety
    /// Caller must guarantee `index` points to a valid sequence of bytes of layout `T`.
    #[inline]
    pub unsafe fn get_mut_unchecked<T: Component>(&mut self, index: usize) -> &mut T {
        &mut *(self.data.add(index * self.item_layout.size()) as *mut T)
    }

    unsafe fn drop_ptr<T>(ptr: *mut u8) {
        ptr::drop_in_place(ptr as *mut T);
    }
}
// ... continuation of archetype.rs ...

impl Archetype {
    pub fn new(id: ArchetypeId) -> Self {
        Self {
            id,
            component_indices: HashMap::new(),
            columns: Vec::new(),
            entities: Vec::new(),
        }
    }
}

impl Drop for Archetype {
    fn drop(&mut self) {
        let len = self.entities.len();
        for column in &mut self.columns {
            if let Some(drop_fn) = column.item_drop_fn {
                for i in 0..len {
                    unsafe {
                        let ptr = column.data.add(i * column.item_layout.size());
                        (drop_fn)(ptr);
                    }
                }
            }
        }
        self.entities.clear();
    }
}

impl Archetype {
    /// Adds a structural Component definition to this Archetype. This allocates an empty internal contiguous buffer.
    pub fn add_column<T: Component>(&mut self) {
        let component_id = ComponentId::of::<T>();
        if !self.component_indices.contains_key(&component_id) {
            let index = self.columns.len();
            self.columns.push(Column::new::<T>(0));
            self.component_indices.insert(component_id, index);
        }
    }

    /// Adds an already-erased empty column definition.
    pub fn add_untyped_column(&mut self, mut empty_col: Column) {
        if !self.component_indices.contains_key(&empty_col.component_id) {
            let index = self.columns.len();
            let comp_id = empty_col.component_id;
            empty_col.capacity = 0; // enforce empty validation
            self.columns.push(empty_col);
            self.component_indices.insert(comp_id, index);
        }
    }

    /// Fetches the index within the chunk for the provided Element.
    pub fn entity_index(&self, entity: Entity) -> Option<usize> {
        self.entities.iter().position(|&e| e == entity)
    }

    /// Returns the length of all contiguous data columns currently active in the structural Archetype.
    pub fn len(&self) -> usize {
        self.entities.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    /// Allocates an entity within this Archetype. Returns the row index.
    pub fn allocate(&mut self, entity: Entity) -> usize {
        let index = self.entities.len();
        self.entities.push(entity);

        // Ensure that all columns have backing capacity for the new entity slot index.
        for column in &mut self.columns {
            if index >= column.capacity {
                unsafe {
                    // Start at 4 or double to grow
                    let new_cap = if column.capacity == 0 {
                        4
                    } else {
                        column.capacity * 2
                    };
                    column.grow(new_cap);
                }
            }
        }

        index
    }

    /// Inserts a typed component instance at the specified `row` index.
    ///
    /// # Safety
    /// The caller must guarantee `row` represents a valid pre-allocated entity slot bounds check.
    pub unsafe fn insert_component<T: Component>(&mut self, row: usize, component: T) {
        if let Some(&col_idx) = self.component_indices.get(&ComponentId::of::<T>()) {
            let column = &mut self.columns[col_idx];
            column.push(row, component);
        } else {
            // Memory leak sanity check
            std::mem::drop(component);
            panic!(
                "Attempted to insert component {:?} into an Archetype that does not define it.",
                std::any::type_name::<T>()
            );
        }
    }

    /// Moves an entity from `self` into the `target` archetype structure by performing type-safe memory copying.
    /// Returns the old Entity row slot swapped from the back in `self` so callers can fix up index tracking.
    ///
    /// # Safety
    /// Callers must guarantee that the destination archetype perfectly mirrors the expected copied offsets.
    pub unsafe fn migrate_entity_to(
        &mut self,
        entity: Entity,
        target: &mut Archetype,
    ) -> Option<(Entity, usize)> {
        if let Some(src_row) = self.entity_index(entity) {
            // Allocate the new slot in the target archetype
            let dst_row = target.allocate(entity);

            // For every component mapped in `target`, copy the raw memory slice from `self`.
            for (comp_id, tgt_col_idx) in &target.component_indices {
                let target_column = &mut target.columns[*tgt_col_idx];

                if let Some(&src_col_idx) = self.component_indices.get(comp_id) {
                    let src_column = &self.columns[src_col_idx];

                    let sz = src_column.item_layout.size();
                    if sz > 0 {
                        let src_ptr = src_column.get_ptr(src_row);
                        let dst_ptr = target_column.data.add(dst_row * sz);

                        ptr::copy_nonoverlapping(src_ptr, dst_ptr, sz);
                    }
                }
                // (Note: if `target` has components `self` lacks, those slots are technically uninitialized bytes
                // and the ECS World system must immediately initialize them after invoking `migrate_entity_to()`).
            }

            // Remove the entity from `self` by performing a fast swap_remove
            self.entities.swap_remove(src_row);

            // Perform identical swap_removes on the type-erased columnar data
            let is_last = src_row == self.entities.len();
            for column in &mut self.columns {
                let sz = column.item_layout.size();
                if sz > 0 {
                    let dst_ptr = column.data.add(src_row * sz);

                    if !is_last {
                        // Swap the last element into `src_row`
                        let src_ptr = column.data.add(self.entities.len() * sz);
                        ptr::copy_nonoverlapping(src_ptr, dst_ptr, sz);
                    }
                }
            }

            // If another entity was shifted down, return it so external trackers can update its index pointer
            if !is_last {
                return Some((self.entities[src_row], src_row));
            }
        }
        None
    }

    /// Safely fetches a reference to a specific type component out of the provided Entity row.
    pub fn get_component<T: Component>(&self, entity: Entity) -> Option<&T> {
        if let Some(row) = self.entity_index(entity) {
            if let Some(&col_idx) = self.component_indices.get(&ComponentId::of::<T>()) {
                unsafe {
                    return Some(self.columns[col_idx].get_unchecked::<T>(row));
                }
            }
        }
        None
    }

    /// Safely fetches a mutable reference to a specific type component out of the provided Entity row.
    pub fn get_component_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T> {
        if let Some(row) = self.entity_index(entity) {
            if let Some(&col_idx) = self.component_indices.get(&ComponentId::of::<T>()) {
                unsafe {
                    return Some(self.columns[col_idx].get_mut_unchecked::<T>(row));
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    struct ComponentA(i32);

    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    struct ComponentB(u32);

    #[test]
    fn archetype_add_column_and_allocate() {
        let mut arch = Archetype::new(ArchetypeId(1));
        arch.add_column::<ComponentA>();

        assert_eq!(arch.columns.len(), 1);

        let e = Entity::new(0, 0);
        let row = arch.allocate(e);
        assert_eq!(row, 0);
        assert_eq!(arch.len(), 1);

        unsafe {
            let col = &mut arch.columns[0];
            col.push(row, ComponentA(42));
        }

        let val = arch.get_component::<ComponentA>(e).unwrap();
        assert_eq!(val.0, 42);
    }

    #[test]
    fn archetype_migration_moves_memory_and_shifts_tail() {
        let mut source = Archetype::new(ArchetypeId(1));
        source.add_column::<ComponentA>();
        source.add_column::<ComponentB>();

        let mut target = Archetype::new(ArchetypeId(2));
        target.add_column::<ComponentA>(); // Target drops ComponentB

        let e1 = Entity::new(1, 0);
        let e2 = Entity::new(2, 0);

        let r1 = source.allocate(e1);
        let r2 = source.allocate(e2);

        unsafe {
            source.columns[0].push(r1, ComponentA(100)); // index 0 might be A or B depending on add order, but we'll lookup safely

            let a_idx = *source
                .component_indices
                .get(&ComponentId::of::<ComponentA>())
                .unwrap();
            let b_idx = *source
                .component_indices
                .get(&ComponentId::of::<ComponentB>())
                .unwrap();

            source.columns[a_idx].push(r1, ComponentA(100));
            source.columns[b_idx].push(r1, ComponentB(200));

            source.columns[a_idx].push(r2, ComponentA(300));
            source.columns[b_idx].push(r2, ComponentB(400));
        }

        // Migrate e1 out of `source`
        unsafe {
            let shift = source.migrate_entity_to(e1, &mut target);
            // e1 was row 0. We pop-swapped row 1 (e2) into row 0.
            assert_eq!(shift, Some((e2, 0)));
        }

        // Validate e1 is successfully in target
        assert_eq!(target.len(), 1);
        assert_eq!(target.get_component::<ComponentA>(e1).unwrap().0, 100);

        // Validate e2 is now at row 0 of source
        assert_eq!(source.len(), 1);
        assert_eq!(source.get_component::<ComponentA>(e2).unwrap().0, 300);
        assert_eq!(source.get_component::<ComponentB>(e2).unwrap().0, 400);
    }
}
