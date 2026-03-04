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
    /// Optional custom runner. Plugins (e.g. WindowPlugin) can replace the
    /// default game loop by installing their own runner via `set_runner()`.
    runner: Option<Box<dyn FnOnce(App)>>,
}

impl Default for App {
    fn default() -> Self {
        let mut app = Self {
            world: World::new(),
            schedule: Schedule::new(),
            runner: None,
        };
        app.add_event::<AppExit>();
        app
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_plugin<P: Plugin>(&mut self, plugin: P) -> &mut Self {
        plugin.build(self);
        self
    }

    pub fn insert_resource<R: Resource>(&mut self, resource: R) -> &mut Self {
        self.world.insert_resource(resource);
        self
    }

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

    /// Install a custom runner that replaces the default game loop.
    /// The runner receives ownership of the `App` and is responsible for
    /// calling `app.update()` each frame.
    pub fn set_runner(&mut self, runner: impl FnOnce(App) + 'static) -> &mut Self {
        self.runner = Some(Box::new(runner));
        self
    }

    /// Run one tick of the ECS schedule and check for AppExit.
    /// Returns `true` if the app should continue, `false` if exit was requested.
    pub fn update(&mut self) -> bool {
        self.schedule.run(&mut self.world);

        if let Some(exit_events) = self.world.resource::<Events<AppExit>>() {
            if !exit_events.events.is_empty() {
                return false;
            }
        }
        true
    }

    /// Enter the application's blocking loop.
    /// If a custom runner was installed (e.g. by WindowPlugin), it takes over.
    /// Otherwise, falls back to a simple loop.
    pub fn run(mut self) {
        if let Some(runner) = self.runner.take() {
            runner(self);
        } else {
            // Default headless loop
            loop {
                if !self.update() {
                    break;
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

    fn exit_after_n_frames(counter: crate::system::ResMut<FrameCounter>, mut exit_events: EventWriter<AppExit>) {
        counter.0.0 += 1;
        if counter.0.0 >= 3 {
            exit_events.send(AppExit);
        }
    }

    #[test]
    fn app_update_runs_and_exits() {
        let mut app = App::new();
        app.insert_resource(FrameCounter(0));
        app.add_system::<fn(crate::system::ResMut<'static, FrameCounter>, EventWriter<'static, AppExit>), _>(exit_after_n_frames);

        // Use update() so we retain access to app for assertions
        while app.update() {}

        assert_eq!(app.world.resource::<FrameCounter>().unwrap().0, 3);
    }

    #[test]
    fn app_run_consumes_and_exits() {
        let mut app = App::new();
        app.insert_resource(FrameCounter(0));
        app.add_system::<fn(crate::system::ResMut<'static, FrameCounter>, EventWriter<'static, AppExit>), _>(exit_after_n_frames);

        // run() consumes self — if it doesn't panic or hang, exit worked
        app.run();
    }
}

