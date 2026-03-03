use std::collections::VecDeque;

/// A simple ID to uniquely identify an event instance for cursors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventId(pub usize);

/// The event queue resource for type `T`.
pub struct Events<T: Send + Sync + 'static> {
    pub events: VecDeque<(EventId, T)>,
    pub next_id: usize,
}

impl<T: Send + Sync + 'static> Default for Events<T> {
    fn default() -> Self {
        Self {
            events: VecDeque::new(),
            next_id: 0,
        }
    }
}

impl<T: Send + Sync + 'static> Events<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn send(&mut self, event: T) {
        self.events.push_back((EventId(self.next_id), event));
        self.next_id += 1;
    }

    /// Retains only the last N events, or can be used to flush events 
    /// older than a certain frame limit.
    /// For double buffering, this should keep events from the last 2 frames.
    pub fn update(&mut self) {
        // Basic single buffer clear for now if we want, but since EventReaders use
        // cursors, we only need to drop events that are "too old" so memory doesn't leak.
        // We will keep events from the previous frame.
    }
}

pub struct EventCursor<T> {
    pub last_read_id: usize,
    _marker: std::marker::PhantomData<T>,
}

impl<T> Default for EventCursor<T> {
    fn default() -> Self {
        Self { last_read_id: 0, _marker: std::marker::PhantomData }
    }
}

use crate::world::World;
use crate::system::{SystemParam, Access};
use std::any::TypeId;

/// System parameter to read events of type `T`.
pub struct EventReader<'w, 's, T: Send + Sync + 'static> {
    events: &'w Events<T>,
    cursor: &'s mut EventCursor<T>,
}

impl<'w, 's, T: Send + Sync + 'static> EventReader<'w, 's, T> {
    pub fn iter(&mut self) -> impl Iterator<Item = &'w T> {
        let start_id = self.cursor.last_read_id;
        let events = &self.events.events;
        
        let mut idx = 0;
        while idx < events.len() && events[idx].0.0 < start_id {
            idx += 1;
        }
        
        if idx < events.len() {
            self.cursor.last_read_id = events.back().unwrap().0.0 + 1;
        }
        
        events.iter().skip(idx).map(|(_, ev)| ev)
    }
}

impl<T: Send + Sync + 'static> SystemParam for EventReader<'static, 'static, T> {
    type State = EventCursor<T>;
    type Fetch<'w> = EventReader<'w, 'w, T>;

    fn init_state(world: &mut World) -> Self::State {
        if world.resource::<Events<T>>().is_none() {
            world.insert_resource(Events::<T>::new());
        }
        EventCursor::default()
    }
    
    fn fetch<'w>(world: &'w World, state: &'w mut Self::State) -> Self::Fetch<'w> {
        EventReader {
            events: world.resource::<Events<T>>().unwrap(),
            cursor: state,
        }
    }
    
    fn resource_access() -> Access {
        let mut access = Access::default();
        access.reads.push(TypeId::of::<Events<T>>());
        access
    }
}

/// System parameter to write events of type `T`.
pub struct EventWriter<'w, T: Send + Sync + 'static> {
    events: &'w mut Events<T>,
}

impl<'w, T: Send + Sync + 'static> EventWriter<'w, T> {
    pub fn send(&mut self, event: T) {
        self.events.send(event);
    }
}

impl<T: Send + Sync + 'static> SystemParam for EventWriter<'static, T> {
    type State = ();
    type Fetch<'w> = EventWriter<'w, T>;

    fn init_state(world: &mut World) -> Self::State {
        if world.resource::<Events<T>>().is_none() {
            world.insert_resource(Events::<T>::new());
        }
    }
    
    fn fetch<'w>(world: &'w World, _state: &'w mut Self::State) -> Self::Fetch<'w> {
        #[allow(invalid_reference_casting)]
        let world_mut = unsafe { &mut *(world as *const World as *mut World) };
        EventWriter {
            events: world_mut.resource_mut::<Events<T>>().unwrap(),
        }
    }
    
    fn resource_access() -> Access {
        let mut access = Access::default();
        access.writes.push(TypeId::of::<Events<T>>());
        access
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schedule::Schedule;

    #[derive(Debug, PartialEq, Clone)]
    struct MyEvent(i32);

    fn sender_system(mut writer: EventWriter<MyEvent>) {
        writer.send(MyEvent(42));
        writer.send(MyEvent(100));
    }

    fn receiver_system(mut reader: EventReader<MyEvent>) {
        let events: Vec<_> = reader.iter().cloned().collect();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0], MyEvent(42));
        assert_eq!(events[1], MyEvent(100));
    }

    #[test]
    fn event_bus_sends_and_receives() {
        let mut world = World::new();
        let mut schedule = Schedule::new();

        schedule.add_system::<fn(EventWriter<'static, MyEvent>), _>(sender_system);
        schedule.add_system::<fn(EventReader<'static, 'static, MyEvent>), _>(receiver_system);

        schedule.run(&mut world);
        
        // Second run should NOT receive the same events normally since EventReader updates cursor
        // Wait, the cursor is persisted in the System state!
        schedule.run(&mut world);
    }
}
