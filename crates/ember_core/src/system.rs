use std::any::TypeId;
use crate::world::World;
use crate::query::Query;
use crate::resource::Resource;

/// Tracks reads and writes for topological sorting and conflict detection.
#[derive(Default, Debug, Clone)]
pub struct Access {
    pub reads: Vec<TypeId>,
    pub writes: Vec<TypeId>,
}

impl Access {
    pub fn conflicts_with(&self, other: &Access) -> bool {
        // Conflict if either writes what the other reads or writes
        for write in &self.writes {
            if other.reads.contains(write) || other.writes.contains(write) {
                return true;
            }
        }
        for write in &other.writes {
            if self.reads.contains(write) || self.writes.contains(write) {
                return true;
            }
        }
        false
    }
    
    pub fn merge(&mut self, other: &Access) {
        for r in &other.reads {
            if !self.reads.contains(r) { self.reads.push(*r); }
        }
        for w in &other.writes {
            if !self.writes.contains(w) { self.writes.push(*w); }
        }
    }
}

pub trait System: Send + Sync {
    /// Check if local state has been initialized.
    fn is_initialized(&self) -> bool;
    /// Initialize any local state required by this system. Called once before the first run.
    fn initialize(&mut self, world: &mut World);
    fn run(&mut self, world: &World);
    fn component_access(&self) -> Access;
    fn resource_access(&self) -> Access;
}

/// A parameter that a System can fetch from the World.
pub trait SystemParam: Sized {
    type State: Send + Sync + 'static;
    type Fetch<'w>;
    
    fn init_state(world: &mut World) -> Self::State;
    fn fetch<'w>(world: &'w World, state: &'w mut Self::State) -> Self::Fetch<'w>;
    
    fn component_access() -> Access { Access::default() }
    fn resource_access() -> Access { Access::default() }
}

/// Represents an immutable borrow of a Resource.
pub struct Res<'w, T: Resource>(pub &'w T);

impl<'w, T: Resource> std::ops::Deref for Res<'w, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

/// Represents a mutable borrow of a Resource.
pub struct ResMut<'w, T: Resource>(pub &'w mut T);

impl<'w, T: Resource> std::ops::Deref for ResMut<'w, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'w, T: Resource> std::ops::DerefMut for ResMut<'w, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
    }
}

// SystemParam for Res<T>
impl<T: Resource> SystemParam for Res<'static, T> {
    type State = ();
    type Fetch<'w> = Res<'w, T>;
    
    fn init_state(_world: &mut World) -> Self::State {}
    fn fetch<'w>(world: &'w World, _state: &'w mut Self::State) -> Self::Fetch<'w> {
        Res(world.resource::<T>().expect("Resource requested by system but not found in World"))
    }
    fn resource_access() -> Access {
        let mut access = Access::default();
        access.reads.push(TypeId::of::<T>());
        access
    }
}

// SystemParam for ResMut<T>
impl<T: Resource> SystemParam for ResMut<'static, T> {
    type State = ();
    type Fetch<'w> = ResMut<'w, T>;
    
    fn init_state(_world: &mut World) -> Self::State {}
    fn fetch<'w>(world: &'w World, _state: &'w mut Self::State) -> Self::Fetch<'w> {
        // SAFETY: The Scheduler ensures systems with conflicting mut accesses don't run concurrently.
        // We cast *const to *mut to bypass the immutable World borrow inside System::run
        #[allow(invalid_reference_casting)]
        let world_mut = unsafe { &mut *(world as *const World as *mut World) };
        ResMut(world_mut.resource_mut::<T>().expect("ResourceMut requested by system but not found in World"))
    }
    fn resource_access() -> Access {
        let mut access = Access::default();
        access.writes.push(TypeId::of::<T>());
        access
    }
}

use crate::query::WorldQuery;

// SystemParam for Query
impl<Q: WorldQuery + 'static> SystemParam for Query<'static, Q> {
    type State = ();
    type Fetch<'w> = Query<'w, Q>;
    
    fn init_state(_world: &mut World) -> Self::State {}
    fn fetch<'w>(world: &'w World, _state: &'w mut Self::State) -> Self::Fetch<'w> {
        let world_mut = unsafe { &*(world as *const World) };
        world_mut.query::<Q>()
    }
    
    fn component_access() -> Access {
        Q::access()
    }
}

pub trait SystemParamFunction<Marker>: Send + Sync + 'static {
    type State: Send + Sync + 'static;
    fn init_state(&self, world: &mut World) -> Self::State;
    fn run(&mut self, world: &World, state: &mut Self::State);
    fn component_access(&self) -> Access;
    fn resource_access(&self) -> Access;
}

pub trait IntoSystem<Marker> {
    type System: System;
    fn into_system(self) -> Self::System;
}

impl<Marker, F> IntoSystem<Marker> for F
where
    Marker: 'static,
    F: SystemParamFunction<Marker>,
{
    type System = FunctionSystem<F, Marker, F::State>;
    fn into_system(self) -> Self::System {
        FunctionSystem {
            func: self,
            state: None,
            _marker: std::marker::PhantomData,
        }
    }
}

pub struct FunctionSystem<F, Marker, State> {
    func: F,
    state: Option<State>,
    _marker: std::marker::PhantomData<fn() -> Marker>,
}

impl<Marker, F> System for FunctionSystem<F, Marker, F::State>
where
    Marker: 'static,
    F: SystemParamFunction<Marker>,
{
    fn is_initialized(&self) -> bool {
        self.state.is_some()
    }

    fn initialize(&mut self, world: &mut World) {
        self.state = Some(self.func.init_state(world));
    }

    fn run(&mut self, world: &World) {
        if let Some(state) = &mut self.state {
            self.func.run(world, state);
        } else {
            panic!("System not initialized before running!");
        }
    }
    fn component_access(&self) -> Access { self.func.component_access() }
    fn resource_access(&self) -> Access { self.func.resource_access() }
}

macro_rules! impl_into_system {
    ($($param:ident),*) => {
        impl<Func, $($param: SystemParam + 'static),*> SystemParamFunction<fn($($param,)*)> for Func
        where
            for<'w> Func: Send + Sync + 'static + FnMut($($param::Fetch<'w>),*),
        {
            type State = ($( $param::State, )*);
            
            #[allow(clippy::unused_unit)]
            fn init_state(&self, _world: &mut World) -> Self::State {
                ($( $param::init_state(_world), )*)
            }

            #[allow(unused_variables)]
            #[allow(non_snake_case)]
            fn run(&mut self, world: &World, state: &mut Self::State) {
                let ($($param,)*) = state;
                (self)(
                    $($param::fetch(world, $param),)*
                );
            }
            
            fn component_access(&self) -> Access {
                #[allow(unused_mut)]
                let mut access = Access::default();
                $( access.merge(&$param::component_access()); )*
                access
            }
            
            fn resource_access(&self) -> Access {
                #[allow(unused_mut)]
                let mut access = Access::default();
                $( access.merge(&$param::resource_access()); )*
                access
            }
        }
    };
}

impl_into_system!();
impl_into_system!(A);
impl_into_system!(A, B);
impl_into_system!(A, B, C);
impl_into_system!(A, B, C, D);
impl_into_system!(A, B, C, D, E);
impl_into_system!(A, B, C, D, E, F);
