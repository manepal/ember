use ember_core::app::{App, AppExit};
use ember_core::event::EventWriter;
use ember_core::plugin::CorePlugin;
use ember_core::system::Res;
use ember_core::query::Query;
use ember_core::time::Time;

// Define components
#[derive(Debug, PartialEq, Clone)]
pub struct Name(&'static str);

#[derive(Debug, PartialEq, Clone)]
pub struct Position(f32, f32);

#[derive(Debug, PartialEq, Clone)]
pub struct Velocity(f32, f32);

// Setup system to spawn entities
fn setup_system(world: &mut ember_core::world::World) {
    println!("Setting up the ECS World!");
    world.spawn()
        .insert(Name("Player"))
        .insert(Position(0.0, 0.0))
        .insert(Velocity(1.0, 2.0));

    world.spawn()
        .insert(Name("Enemy"))
        .insert(Position(10.0, 10.0))
        .insert(Velocity(-1.0, -1.0));
}

// System to move entities
fn physics_system(query: Query<(&'static mut Position, &'static Velocity)>, time: Res<Time>) {
    let dt = time.delta_seconds();
    for (pos, vel) in query.iter() {
        pos.0 += vel.0 * dt;
        pos.1 += vel.1 * dt;
    }
}

// System to print entity states and trigger exit after 3 frames
fn print_system(
    query: Query<(&'static Name, &'static Position)>,
    time: Res<Time>,
    mut exit: EventWriter<AppExit>,
) {
    println!("\n--- Frame {} (Elapsed: {:.4}s) ---", time.frame_count(), time.elapsed_seconds());
    for (name, pos) in query.iter() {
        println!("{}: at ({:.2}, {:.2})", name.0, pos.0, pos.1);
    }

    if time.frame_count() >= 3 {
        println!("Demo complete. Emitting AppExit.");
        exit.send(AppExit);
    }
}

// System to simulate time passing (since no Window or external loop controls delta here)
fn time_update_system(mut time: ember_core::system::ResMut<Time>) {
    time.update(std::time::Duration::from_millis(16));
}

fn main() {
    println!("Starting Ember Engine Demo...");
    
    let mut app = App::new();
    
    // CorePlugin provides the Time resource
    app.add_plugin(CorePlugin);
    
    // Add our systems
    app.add_system::<fn(ember_core::system::ResMut<'static, Time>), _>(time_update_system);
    app.add_system::<fn(Query<'static, (&'static mut Position, &'static Velocity)>, Res<'static, Time>), _>(physics_system);
    app.add_system::<fn(Query<'static, (&'static Name, &'static Position)>, Res<'static, Time>, EventWriter<'static, AppExit>), _>(print_system);

    // Run custom setup logic (since we don't have Startup systems yet)
    setup_system(&mut app.world);

    // Run the main game loop (consumes app)
    app.run();
}
