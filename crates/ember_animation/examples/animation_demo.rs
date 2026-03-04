//! Animation Demo — Demonstrates sprite animation, tweening, and state machines
//!
//! Run: `cargo run --example animation_demo -p ember_animation`

use ember_animation::sprite_animation::{SpriteAnimationClip, SpriteAnimator};
use ember_animation::state_machine::{
    AnimTransition, AnimationEvent, AnimationStateMachine, TransitionCondition,
};
use ember_animation::tween::{easing, Tween, TweenMode, TweenSequence};

fn main() {
    println!("=== Ember Animation Demo ===\n");

    // ─── Sprite Animation ───
    println!("--- Sprite Sheet Animation ---");
    let idle = SpriteAnimationClip::new("idle", vec![0, 1, 2, 3], 0.2, true);
    let run = SpriteAnimationClip::new("run", vec![4, 5, 6, 7, 8, 9], 0.1, true);
    let attack = SpriteAnimationClip::new("attack", vec![10, 11, 12, 13, 14], 0.08, false);

    let mut animator = SpriteAnimator::new(vec![idle, run, attack], "idle");

    println!("Playing 'idle' clip:");
    for i in 0..12 {
        let frame = animator.update(0.2);
        println!("  t={:.1}s → atlas frame {:?}", (i + 1) as f32 * 0.2, frame);
    }

    println!("\nSwitching to 'run' clip:");
    animator.set_clip("run");
    for i in 0..8 {
        let frame = animator.update(0.1);
        println!("  t={:.1}s → atlas frame {:?}", (i + 1) as f32 * 0.1, frame);
    }

    println!("\nPlaying 'attack' (non-looping):");
    animator.set_clip("attack");
    for i in 0..8 {
        let frame = animator.update(0.08);
        println!(
            "  t={:.2}s → atlas frame {:?}  state={:?}",
            (i + 1) as f32 * 0.08,
            frame,
            animator.state
        );
    }

    // ─── Tweening ───
    println!("\n--- Tweening ---");

    // Linear tween
    let mut tween = Tween::new(0.0, 100.0, 1.0);
    println!("Linear tween 0→100 over 1s:");
    for i in 1..=5 {
        let val = tween.update(0.2);
        println!("  t={:.1}s → {:.1}", i as f32 * 0.2, val);
    }

    // Ease-in-out with bounce
    println!("\nBounce-out tween 0→200 over 1s:");
    let mut bounce = Tween::new(0.0, 200.0, 1.0).with_easing(easing::bounce_out);
    for i in 1..=10 {
        let val = bounce.update(0.1);
        println!("  t={:.1}s → {:.1}", i as f32 * 0.1, val);
    }

    // PingPong tween
    println!("\nPingPong tween 0→50 (2 cycles):");
    let mut pp = Tween::new(0.0, 50.0, 0.5).with_mode(TweenMode::PingPong);
    for i in 1..=10 {
        let val = pp.update(0.15);
        println!("  t={:.2}s → {:.1}", i as f32 * 0.15, val);
    }

    // Tween Sequence
    println!("\nTween Sequence (chain 3 tweens):");
    let mut seq = TweenSequence::new(vec![
        Tween::new(0.0, 10.0, 0.5),
        Tween::new(10.0, -5.0, 0.3).with_easing(easing::ease_out_quad),
        Tween::new(-5.0, 20.0, 0.4).with_easing(easing::elastic_out),
    ]);
    for i in 1..=12 {
        let val = seq.update(0.1);
        println!(
            "  t={:.1}s → {:.2}  step={}  finished={}",
            i as f32 * 0.1,
            val,
            seq.current_index,
            seq.finished
        );
    }

    // ─── Animation State Machine ───
    println!("\n--- Animation State Machine ---");

    let mut sm = AnimationStateMachine::new("idle");

    sm.add_transition(AnimTransition::new(
        "idle",
        "walk",
        vec![TransitionCondition::FloatGreater("speed".into(), 0.1)],
    ));
    sm.add_transition(AnimTransition::new(
        "walk",
        "run",
        vec![TransitionCondition::FloatGreater("speed".into(), 5.0)],
    ));
    sm.add_transition(AnimTransition::new(
        "walk",
        "idle",
        vec![TransitionCondition::FloatLess("speed".into(), 0.1)],
    ));
    sm.add_transition(AnimTransition::new(
        "idle",
        "attack",
        vec![TransitionCondition::TriggerSet("do_attack".into())],
    ));

    // Add frame events
    sm.add_event(AnimationEvent {
        clip_name: "walk".into(),
        frame_index: 2,
        tag: "footstep_left".into(),
    });
    sm.add_event(AnimationEvent {
        clip_name: "walk".into(),
        frame_index: 5,
        tag: "footstep_right".into(),
    });

    // Simulate state changes
    println!("Initial state: {}", sm.current_state);

    sm.set_float("speed", 3.0);
    let transition = sm.evaluate();
    println!(
        "Set speed=3.0 → transition={:?}, state={}",
        transition, sm.current_state
    );

    // Check for walk events
    let events = sm.check_events("walk", 2);
    println!(
        "Walk frame 2 events: {:?}",
        events.iter().map(|e| &e.tag).collect::<Vec<_>>()
    );

    sm.set_float("speed", 7.0);
    let transition = sm.evaluate();
    println!(
        "Set speed=7.0 → transition={:?}, state={}",
        transition, sm.current_state
    );

    sm.set_float("speed", 0.0);
    let transition = sm.evaluate();
    println!(
        "Set speed=0.0 → transition={:?} (no matching from 'run')",
        transition
    );

    println!("\n=== Demo Complete ===");
}
