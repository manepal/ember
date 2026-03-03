use crate::system::{IntoSystem, System};
use crate::world::World;

/// A collection of Systems meant to be executed together against the ECS World natively.
#[derive(Default)]
pub struct Schedule {
    systems: Vec<Box<dyn System>>,
}

impl Schedule {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_system<M: 'static, S: IntoSystem<M>>(&mut self, system: S) 
    where
        S::System: 'static,
    {
        self.systems.push(Box::new(system.into_system()));
    }

    /// Runs all registered Systems sequentially against the World.
    /// In future milestones this will generate a parallel dependency graph!
    pub fn run(&mut self, world: &mut World) {
        for system in &mut self.systems {
            if !system.is_initialized() {
                system.initialize(world);
            }
            system.run(world);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::Query;
    use crate::system::Res;

    #[derive(Debug, PartialEq)]
    struct Position(f32, f32);

    #[derive(Debug, PartialEq)]
    struct Velocity(f32, f32);

    #[derive(Debug, PartialEq)]
    struct TimeDelta(f32);

    #[test]
    fn schedule_runs_system_closure() {
        let mut world = World::new();
        world.spawn().insert(Position(0.0, 0.0)).insert(Velocity(1.0, 2.0)).id();
        world.spawn().insert(Position(5.0, -5.0)).insert(Velocity(-1.0, 1.0)).id();
        
        world.insert_resource(TimeDelta(0.5));

        let mut schedule = Schedule::new();
        
        fn physics_system(mut query: Query<(&'static mut Position, &'static Velocity)>, time: Res<TimeDelta>) {
            for (pos, vel) in query.iter() {
                pos.0 += vel.0 * time.0.0;
                pos.1 += vel.1 * time.0.0;
            }
        }
        
        schedule.add_system::<fn(Query<'static, (&'static mut Position, &'static Velocity)>, Res<'static, TimeDelta>), _>(physics_system);

        schedule.run(&mut world);

        let query = world.query::<&Position>();
        let positions: Vec<_> = query.iter().collect();
        assert_eq!(positions.len(), 2);
        assert!(positions.contains(&&Position(0.5, 1.0)));
        assert!(positions.contains(&&Position(4.5, -4.5)));
    }
}
