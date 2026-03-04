use std::collections::HashMap;

/// Parameter types used for transition conditions in the animation state machine.
#[derive(Debug, Clone)]
pub enum AnimParam {
    Bool(bool),
    Float(f32),
    /// A trigger that auto-resets after being consumed.
    Trigger(bool),
}

/// Condition for a state machine transition.
#[derive(Debug, Clone)]
pub enum TransitionCondition {
    /// Transition when a bool parameter is true.
    BoolTrue(String),
    /// Transition when a bool parameter is false.
    BoolFalse(String),
    /// Transition when a float parameter exceeds a threshold.
    FloatGreater(String, f32),
    /// Transition when a float parameter is below a threshold.
    FloatLess(String, f32),
    /// Transition when a trigger parameter is set.
    TriggerSet(String),
}

/// A transition between two animation states.
#[derive(Debug, Clone)]
pub struct AnimTransition {
    pub from: String,
    pub to: String,
    pub conditions: Vec<TransitionCondition>,
}

impl AnimTransition {
    pub fn new(from: &str, to: &str, conditions: Vec<TransitionCondition>) -> Self {
        Self {
            from: from.to_string(),
            to: to.to_string(),
            conditions,
        }
    }
}

/// An animation event fired when playback reaches a specific frame.
#[derive(Debug, Clone)]
pub struct AnimationEvent {
    /// The clip name this event belongs to.
    pub clip_name: String,
    /// Frame index that triggers this event.
    pub frame_index: usize,
    /// Event tag for the ECS to match on.
    pub tag: String,
}

/// Fired as an ECS event when an animation frame event triggers.
#[derive(Debug, Clone)]
pub struct AnimationEventFired {
    pub tag: String,
    pub clip_name: String,
    pub frame_index: usize,
}

/// Component that manages animation state transitions based on parameters.
#[derive(Debug, Clone)]
pub struct AnimationStateMachine {
    pub current_state: String,
    pub transitions: Vec<AnimTransition>,
    pub parameters: HashMap<String, AnimParam>,
    pub events: Vec<AnimationEvent>,
}

impl AnimationStateMachine {
    pub fn new(initial_state: &str) -> Self {
        Self {
            current_state: initial_state.to_string(),
            transitions: Vec::new(),
            parameters: HashMap::new(),
            events: Vec::new(),
        }
    }

    pub fn add_transition(&mut self, transition: AnimTransition) {
        self.transitions.push(transition);
    }

    pub fn set_bool(&mut self, name: &str, value: bool) {
        self.parameters
            .insert(name.to_string(), AnimParam::Bool(value));
    }

    pub fn set_float(&mut self, name: &str, value: f32) {
        self.parameters
            .insert(name.to_string(), AnimParam::Float(value));
    }

    pub fn set_trigger(&mut self, name: &str) {
        self.parameters
            .insert(name.to_string(), AnimParam::Trigger(true));
    }

    pub fn add_event(&mut self, event: AnimationEvent) {
        self.events.push(event);
    }

    /// Evaluate all transitions from the current state. Returns the new state name
    /// if a transition fires, or None if no transition conditions are met.
    pub fn evaluate(&mut self) -> Option<String> {
        let current = &self.current_state;

        for transition in &self.transitions {
            if transition.from != *current {
                continue;
            }

            let all_met = transition.conditions.iter().all(|cond| match cond {
                TransitionCondition::BoolTrue(name) => {
                    matches!(self.parameters.get(name), Some(AnimParam::Bool(true)))
                }
                TransitionCondition::BoolFalse(name) => {
                    matches!(self.parameters.get(name), Some(AnimParam::Bool(false)))
                }
                TransitionCondition::FloatGreater(name, threshold) => {
                    matches!(self.parameters.get(name), Some(AnimParam::Float(v)) if *v > *threshold)
                }
                TransitionCondition::FloatLess(name, threshold) => {
                    matches!(self.parameters.get(name), Some(AnimParam::Float(v)) if *v < *threshold)
                }
                TransitionCondition::TriggerSet(name) => {
                    matches!(self.parameters.get(name), Some(AnimParam::Trigger(true)))
                }
            });

            if all_met {
                let new_state = transition.to.clone();

                // Consume triggers
                for cond in &transition.conditions {
                    if let TransitionCondition::TriggerSet(name) = cond {
                        self.parameters
                            .insert(name.clone(), AnimParam::Trigger(false));
                    }
                }

                self.current_state = new_state.clone();
                return Some(new_state);
            }
        }

        None
    }

    /// Check if any animation events should fire for the given clip and frame.
    pub fn check_events(&self, clip_name: &str, frame_index: usize) -> Vec<AnimationEventFired> {
        self.events
            .iter()
            .filter(|e| e.clip_name == clip_name && e.frame_index == frame_index)
            .map(|e| AnimationEventFired {
                tag: e.tag.clone(),
                clip_name: e.clip_name.clone(),
                frame_index: e.frame_index,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transition_on_bool() {
        let mut sm = AnimationStateMachine::new("idle");
        sm.add_transition(AnimTransition::new(
            "idle",
            "walk",
            vec![TransitionCondition::BoolTrue("moving".into())],
        ));

        // Not moving yet
        assert!(sm.evaluate().is_none());
        assert_eq!(sm.current_state, "idle");

        // Start moving
        sm.set_bool("moving", true);
        assert_eq!(sm.evaluate(), Some("walk".to_string()));
        assert_eq!(sm.current_state, "walk");
    }

    #[test]
    fn transition_on_float() {
        let mut sm = AnimationStateMachine::new("walk");
        sm.add_transition(AnimTransition::new(
            "walk",
            "run",
            vec![TransitionCondition::FloatGreater("speed".into(), 5.0)],
        ));

        sm.set_float("speed", 3.0);
        assert!(sm.evaluate().is_none());

        sm.set_float("speed", 6.0);
        assert_eq!(sm.evaluate(), Some("run".to_string()));
    }

    #[test]
    fn trigger_consumed_after_use() {
        let mut sm = AnimationStateMachine::new("idle");
        sm.add_transition(AnimTransition::new(
            "idle",
            "attack",
            vec![TransitionCondition::TriggerSet("attack".into())],
        ));

        sm.set_trigger("attack");
        assert_eq!(sm.evaluate(), Some("attack".to_string()));

        // Trigger should be consumed — back to idle won't re-trigger
        assert!(matches!(
            sm.parameters.get("attack"),
            Some(AnimParam::Trigger(false))
        ));
    }

    #[test]
    fn animation_events_fire() {
        let mut sm = AnimationStateMachine::new("walk");
        sm.add_event(AnimationEvent {
            clip_name: "walk".into(),
            frame_index: 2,
            tag: "footstep".into(),
        });

        let events = sm.check_events("walk", 2);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].tag, "footstep");

        let events = sm.check_events("walk", 0);
        assert!(events.is_empty());
    }

    #[test]
    fn multiple_conditions_all_required() {
        let mut sm = AnimationStateMachine::new("idle");
        sm.add_transition(AnimTransition::new(
            "idle",
            "dash",
            vec![
                TransitionCondition::BoolTrue("grounded".into()),
                TransitionCondition::TriggerSet("dash".into()),
            ],
        ));

        // Only trigger set — not enough
        sm.set_trigger("dash");
        assert!(sm.evaluate().is_none());

        // Both conditions met
        sm.set_bool("grounded", true);
        sm.set_trigger("dash");
        assert_eq!(sm.evaluate(), Some("dash".to_string()));
    }
}
