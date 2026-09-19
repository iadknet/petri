use super::WorldAction;

/// A bounded queue of world actions that a creature builds during VM execution.
///
/// The queue enforces a hard cap on the number of actions. Pushes beyond the cap
/// are silently ignored (junk-DNA safe). All readback methods return soft defaults
/// (0.0) for out-of-bounds access.
#[derive(Debug, Clone)]
pub struct ActionQueue {
    actions: Vec<WorldAction>,
    cap: usize,
}

impl ActionQueue {
    #[inline]
    pub fn new(cap: usize) -> Self {
        Self {
            actions: Vec::with_capacity(cap.min(16)),
            cap,
        }
    }

    /// Push an action onto the queue. Silent no-op if at cap.
    #[inline]
    pub fn push(&mut self, action: WorldAction) {
        if self.actions.len() < self.cap {
            self.actions.push(action);
        }
    }

    /// Remove and return the last action from the queue.
    #[inline]
    pub fn pop(&mut self) -> Option<WorldAction> {
        self.actions.pop()
    }

    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.actions.len()
    }

    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.actions.is_empty()
    }

    /// Return the action type discriminant at `index` as f32, or 0.0 if OOB.
    #[inline]
    #[must_use]
    pub fn action_type_at(&self, index: usize) -> f32 {
        self.actions
            .get(index)
            .map_or(0.0, |a| a.action_type() as f32)
    }

    /// Return the action parameter at `index` and `slot`, or 0.0 if OOB.
    #[inline]
    #[must_use]
    pub fn param_at(&self, index: usize, slot: usize) -> f32 {
        self.actions.get(index).map_or(0.0, |a| a.param(slot))
    }

    /// Consume the queue and return the accumulated actions.
    #[inline]
    #[must_use]
    pub fn into_actions(self) -> Vec<WorldAction> {
        self.actions
    }

    /// Consume the queue and return the accumulated actions, or `vec![NoOp]` if empty.
    #[inline]
    #[must_use]
    pub fn into_actions_or_noop(self) -> Vec<WorldAction> {
        if self.actions.is_empty() {
            vec![WorldAction::NoOp]
        } else {
            self.actions
        }
    }

    /// Clear all actions from the queue.
    pub fn clear(&mut self) {
        self.actions.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::Direction;
    use crate::contracts::OrdinaryFoodTypeId;

    #[test]
    fn push_within_cap() {
        let mut q = ActionQueue::new(10);
        q.push(WorldAction::eat(OrdinaryFoodTypeId::default()));
        assert_eq!(q.len(), 1);
        q.push(WorldAction::NoOp);
        assert_eq!(q.len(), 2);
    }

    #[test]
    fn push_at_cap_silently_ignored() {
        let mut q = ActionQueue::new(2);
        q.push(WorldAction::eat(OrdinaryFoodTypeId::default()));
        q.push(WorldAction::NoOp);
        q.push(WorldAction::eat(OrdinaryFoodTypeId::default())); // should be ignored
        assert_eq!(q.len(), 2);
    }

    #[test]
    fn pop_removes_last() {
        let mut q = ActionQueue::new(10);
        q.push(WorldAction::eat(OrdinaryFoodTypeId::default()));
        q.push(WorldAction::Move(Direction::N));
        let popped = q.pop();
        assert_eq!(popped, Some(WorldAction::Move(Direction::N)));
        assert_eq!(q.len(), 1);
    }

    #[test]
    fn pop_empty_returns_none() {
        let mut q = ActionQueue::new(10);
        assert_eq!(q.pop(), None);
    }

    #[test]
    fn action_type_at_in_range() {
        let mut q = ActionQueue::new(10);
        q.push(WorldAction::NoOp); // type 0
        q.push(WorldAction::eat(OrdinaryFoodTypeId::default())); // type 1
        q.push(WorldAction::Move(Direction::N)); // type 2
        assert!((q.action_type_at(0) - 0.0).abs() < f32::EPSILON);
        assert!((q.action_type_at(1) - 1.0).abs() < f32::EPSILON);
        assert!((q.action_type_at(2) - 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn action_type_at_out_of_range() {
        let q = ActionQueue::new(10);
        assert!((q.action_type_at(0) - 0.0).abs() < f32::EPSILON);
        assert!((q.action_type_at(99) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn param_at_direction() {
        let mut q = ActionQueue::new(10);
        q.push(WorldAction::Move(Direction::E)); // E = index 2
        assert!((q.param_at(0, 0) - 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn param_at_energy() {
        let mut q = ActionQueue::new(10);
        q.push(WorldAction::Reproduce {
            direction: Direction::S,
            energy_transfer_fraction: 15.0,
        });
        assert!((q.param_at(0, 1) - 15.0).abs() < f32::EPSILON);
    }

    #[test]
    fn param_at_out_of_range() {
        let mut q = ActionQueue::new(10);
        q.push(WorldAction::eat(OrdinaryFoodTypeId::default()));
        assert!((q.param_at(99, 0) - 0.0).abs() < f32::EPSILON); // OOB index
        assert!((q.param_at(0, 99) - 0.0).abs() < f32::EPSILON); // OOB slot
    }

    #[test]
    fn into_actions_or_noop_empty_returns_noop() {
        let q = ActionQueue::new(10);
        let actions = q.into_actions_or_noop();
        assert_eq!(actions, vec![WorldAction::NoOp]);
    }

    #[test]
    fn into_actions_or_noop_non_empty_returns_actions() {
        let mut q = ActionQueue::new(10);
        q.push(WorldAction::eat(OrdinaryFoodTypeId::default()));
        q.push(WorldAction::Move(Direction::N));
        let actions = q.into_actions_or_noop();
        assert_eq!(
            actions,
            vec![
                WorldAction::eat(OrdinaryFoodTypeId::default()),
                WorldAction::Move(Direction::N)
            ]
        );
    }

    #[test]
    fn into_actions_returns_vec() {
        let mut q = ActionQueue::new(10);
        q.push(WorldAction::eat(OrdinaryFoodTypeId::default()));
        q.push(WorldAction::Move(Direction::N));
        q.push(WorldAction::NoOp);
        let actions = q.into_actions();
        assert_eq!(actions.len(), 3);
        assert_eq!(actions[0], WorldAction::eat(OrdinaryFoodTypeId::default()));
        assert_eq!(actions[1], WorldAction::Move(Direction::N));
        assert_eq!(actions[2], WorldAction::NoOp);
    }
}
