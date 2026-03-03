use crate::world::World;
use crate::schedule::Schedule;
use crate::system::IntoSystem;
use crate::resource::Resource;
use crate::plugin::Plugin;
use crate::event::Events;

/// Core event to signal the main `App::run` loop to shut down and exit.
#[derive(Debug, Clone)]
pub struct AppExit;

pub struct App {
    pub world: World,
    pub schedule: Schedule,
}

impl Default for App {
    fn default() -> Self {
        let mut app = Self {
            world: World::new(),
            schedule: Schedule::new(),
        };
        // Ensure AppExit events can be dispatched and handled from systems
        app.add_event::<AppExit>();
        app
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a generic Plugin builder strategy and applies its logic.
    pub fn add_plugin<P: Plugin>(&mut self, plugin: P) -> &mut Self {
        plugin.build(self);
        self
    }

    /// Extends the global `World` memory with a new Singleton Resource.
    pub fn insert_resource<R: Resource>(&mut self, resource: R) -> &mut Self {
        self.world.insert_resource(resource);
        self
    }

    /// Adds a structural ECS Event type by injecting the `Events<T>` queue tracking resource into the Core World.
    pub fn add_event<T: Send + Sync + 'static>(&mut self) -> &mut Self {
        self.insert_resource(Events::<T>::new());
        self
    }

    pub fn add_system<M: 'static, S: IntoSystem<M>>(&mut self, system: S) -> &mut Self
    where
        <S as IntoSystem<M>>::System: 'static,
    {
        self.schedule.add_system(system);
        self
    }

    /// Enters the application's core blocking loop, executing sequentially scheduled `Systems` iteratively 
    /// until an `AppExit` system payload triggers a graceful application break step.
    pub fn run(&mut self) {
        loop {
            self.schedule.run(&mut self.world);
            
            if let Some(exit_events) = self.world.resource::<Events<AppExit>>() {
                if !exit_events.events.is_empty() {
                    break; // Application has received an exit signal
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::EventWriter;

    struct FrameCounter(u32);

    fn exit_after_n_frames(mut counter: crate::system::ResMut<FrameCounter>, mut exit_events: EventWriter<AppExit>) {
        counter.0.0 += 1;
        if counter.0.0 >= 3 {
            exit_events.send(AppExit);
        }
    }

    #[test]
    fn app_runs_loop_and_exits() {
        let mut app = App::new();
        app.insert_resource(FrameCounter(0));
        app.add_system::<fn(crate::system::ResMut<'static, FrameCounter>, EventWriter<'static, AppExit>), _>(exit_after_n_frames);
        
        // This will block indefinitely if `AppExit` logic fails
        app.run();
        
        assert_eq!(app.world.resource::<FrameCounter>().unwrap().0, 3);
    }
}
