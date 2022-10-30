#![allow(unused_variables)] // TODO: Remove

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

mod ranked_pairs;
use ranked_pairs::ranked_pairs_ordering;

mod simple_athens_space;
pub use simple_athens_space::*;

/// Permenant unique identifier for a user.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct UserId(usize);

/// Permenant unique identifier for a task.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TaskId(usize);

/// Globally unique identifier for a space of tasks and users.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SpaceId(usize);

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
    id: UserId,
    alias: String,
    weight: u32,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Task {
    id: TaskId,
    text: String,
}

/// Iterate over TaskId, contains all tasks in the space.
// TODO: A doubly linked list would be more efficient for random reordering.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct OrderedTasks(Vec<TaskId>);

/// Grants parallel access to an AthensSpace
pub trait AthensSpace {
    // Space
    fn id(&self) -> SpaceId;
    // TODO: Get/Set space alias
    fn tasks(&self) -> Vec<TaskId>;
    fn important_tasks(&self) -> Vec<TaskId>;
    fn easy_tasks(&self) -> Vec<TaskId>;
    fn important_and_easy_tasks(&self) -> Vec<TaskId>;
    fn users(&self) -> Vec<UserId>;

    // Modify users and tasks.
    fn create_user(&self) -> User;
    fn set_user(&self, user: User) -> Option<User>;
    fn create_task(&self) -> Task;
    fn set_task(&self, task: Task) -> Option<Task>;

    // Per user
    fn user_importance(&self, id: UserId) -> OrderedTasks;
    fn user_easiness(&self, id: UserId) -> OrderedTasks;
    fn set_user_importance(&self, id: UserId, ord: OrderedTasks) -> Option<OrderedTasks>;
    fn set_user_easiness(&self, id: UserId, ord: OrderedTasks) -> Option<OrderedTasks>;
}

impl std::fmt::Debug for TaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "task/{}", self.0)
    }
}

impl OrderedTasks {
    pub fn from_vec(ids: impl Into<Vec<TaskId>>) -> Self {
        let ids = ids.into();
        let n = ids.iter().collect::<BTreeSet<_>>().len();
        assert_eq!(n, ids.len()); // TODO: Result type
        Self(ids)
    }
    pub fn push_front(&mut self, id: TaskId) {
        debug_assert!(!self.0.contains(&id));
        self.0.insert(0, id);
    }
    pub fn reorder(&mut self, from: usize, to: usize) {
        if from == to {
            return;
        }
        let task = self.0.remove(from);
        self.0.insert(to, task);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn simple_athens_space(space: &dyn AthensSpace) {
        let t0 = space.create_task().id;
        let t1 = space.create_task().id;
        let t2 = space.create_task().id;
        let t3 = space.create_task().id;
        let u0 = space.create_user().id;
        let u1 = space.create_user().id;
        let u2 = space.create_user().id;
        let u3 = space.create_user().id;
        let u4 = space.create_user().id;
        space.set_user_importance(u0, OrderedTasks::from_vec([t0, t1, t3, t2]));
        space.set_user_importance(u1, OrderedTasks::from_vec([t1, t3, t2, t0]));
        space.set_user_importance(u2, OrderedTasks::from_vec([t0, t1, t3, t2]));
        space.set_user_importance(u3, OrderedTasks::from_vec([t0, t3, t2, t1]));
        space.set_user_importance(u4, OrderedTasks::from_vec([t0, t2, t3, t1]));
        // Win margin (row - column)
        //
        //      t1  t2  t3
        // t0    3   3   3
        // t1        1   1
        // t2           -3
        //
        // t0 > all
        // t3 > t2
        // t1 > t3
        // t1 > t2
        assert_eq!(&space.important_tasks(), &[t0, t1, t3, t2]);
    }
    #[test]
    fn test_parallel_simple_athens_space() {
        use std::sync::{Arc, Mutex};
        let s = Arc::new(Mutex::new(SimpleAthensSpace::new()));
        simple_athens_space(&s);
    }
}
