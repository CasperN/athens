#![allow(unused_variables)] // TODO: Remove

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

mod ranked_pairs;
use ranked_pairs::ranked_pairs_ordering;

mod simple_athens_space;
pub use simple_athens_space::*;

/// Permenant unique identifier for a user.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct UserId(pub usize);

/// Permenant unique identifier for a task.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TaskId(pub usize);

/// Globally unique identifier for a space of tasks and users.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SpaceId(usize);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub alias: String,
    pub weight: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Task {
    pub id: TaskId,
    pub text: String,
}

/// Iterate over TaskId, contains all tasks in the space.
// TODO: A doubly linked list would be more efficient for random reordering.
// TODO: Perhaps this shouldn't force copies?
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct OrderedTasks(Vec<TaskId>);

/// Grants parallel access to an AthensSpace
// TODO: Maybe things will be easier if it wasn't parallel.
pub trait AthensSpace {
    // Space
    fn id(&self) -> SpaceId;
    // TODO: Get/Set space alias

    fn tasks(&self) -> Vec<TaskId>; // TODO: does this make sense?
    fn important_tasks(&self) -> OrderedTasks;
    fn easy_tasks(&self) -> OrderedTasks;
    fn important_and_easy_tasks(&self) -> OrderedTasks;
    fn users(&self) -> Vec<UserId>;

    // Crud for users
    fn create_user(&self) -> User;
    fn get_user(&self, user: UserId) -> Option<User>;
    fn set_user(&self, id: User) -> Option<User>;

    // Crud for tasks
    fn create_task(&self) -> Task;
    fn get_task(&self, id: TaskId) -> Option<Task>;
    fn set_task(&self, task: Task) -> Option<Task>;

    // Get per user task ordering
    fn user_importance(&self, id: UserId) -> OrderedTasks;
    fn user_easiness(&self, id: UserId) -> OrderedTasks;
    fn user_important_and_easy(&self, id: UserId) -> OrderedTasks;

    // Modify user orderings
    fn set_user_importance(&self, id: UserId, ord: OrderedTasks) -> Option<OrderedTasks>;
    fn set_user_easiness(&self, id: UserId, ord: OrderedTasks) -> Option<OrderedTasks>;
    fn swap_user_importance(&self, id: UserId, from: usize, to: usize) -> Option<OrderedTasks>;
    fn swap_user_easiness(&self, id: UserId, from: usize, to: usize) -> Option<OrderedTasks>;
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
    pub fn iter(&self) -> impl Iterator<Item = TaskId> + '_ {
        self.0.iter().copied()
    }
}
impl IntoIterator for OrderedTasks {
    type Item = TaskId;
    type IntoIter = std::vec::IntoIter<TaskId>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
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
        let s = Arc::new(Mutex::new(SimpleAthensSpace::new()));
        simple_athens_space(&s);
    }
    // Test list importance with no users and no tasks
    #[test]
    fn test_empty_simple_athens_space() {
        let s = Arc::new(Mutex::new(SimpleAthensSpace::new()));
        assert_eq!(s.important_tasks(), vec![]);
    }
    #[test]
    fn test_simple_athens_space_no_users() {
        let s = Arc::new(Mutex::new(SimpleAthensSpace::new()));
        let t0 = s.create_task().id;
        assert_eq!(s.important_tasks(), vec![t0]);
    }
}
