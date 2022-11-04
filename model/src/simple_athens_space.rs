#![allow(unused_variables)] // TODO: Remove

use super::*; // TODO: explicit import
use std::collections::BTreeMap;
pub type ParallelSimpleAthensSpace = std::sync::Arc<std::sync::Mutex<SimpleAthensSpace>>;

impl AthensSpace for ParallelSimpleAthensSpace {
    fn id(&self) -> SpaceId {
        self.lock().unwrap().id
    }
    fn tasks(&self) -> Vec<TaskId> {
        self.lock().unwrap().tasks.iter().map(|i| i.id).collect()
    }
    fn important_tasks(&self) -> Vec<TaskId> {
        self.lock().unwrap().importance().0
    }
    fn easy_tasks(&self) -> Vec<TaskId> {
        self.lock().unwrap().easiness().0
    }
    fn important_and_easy_tasks(&self) -> Vec<TaskId> {
        let space = self.lock().unwrap();
        let i = space.importance();
        let e = space.easiness();
        combine_important_and_easy(&i, &e).0
    }
    fn create_user(&self) -> User {
        self.lock().unwrap().new_user().user.clone()
    }
    fn create_task(&self) -> Task {
        self.lock().unwrap().new_task().clone()
    }
    fn set_user(&self, user: User) -> Option<User> {
        self.lock().unwrap().mut_user(user.id).map(|u| {
            u.user = user;
            u.user.clone()
        })
    }
    fn get_user(&self, id: UserId) -> Option<User> {
        // TODO: This will panic probably on unknown id
        Some(self.lock().unwrap().user(id).user.clone())
    }
    fn get_task(&self, id: TaskId) -> Option<Task> {
        Some(self.lock().unwrap().task(id).clone())
    }

    fn set_task(&self, task: Task) -> Option<Task> {
        self.lock().unwrap().mut_task(task.id).map(|t| {
            *t = task;
            t.clone()
        })
    }
    fn users(&self) -> Vec<UserId> {
        self.lock()
            .unwrap()
            .users
            .iter()
            .map(|user| user.user.id)
            .collect()
    }
    fn user_importance(&self, id: UserId) -> OrderedTasks {
        self.lock().unwrap().user(id).importance.clone()
    }
    fn user_easiness(&self, id: UserId) -> OrderedTasks {
        self.lock().unwrap().user(id).easiness.clone()
    }
    fn user_important_and_easy(&self, id: UserId) -> OrderedTasks {
        let space = self.lock().unwrap();
        let user = space.user(id);
        combine_important_and_easy(&user.importance, &user.easiness)
   }
    fn set_user_importance(&self, id: UserId, o: OrderedTasks) -> Option<OrderedTasks> {
        // TODO: Verification of taskIds.
        self.lock().unwrap().mut_user(id).map(|u| {
            u.importance = o;
            u.importance.clone()
        })
    }
    fn set_user_easiness(&self, id: UserId, o: OrderedTasks) -> Option<OrderedTasks> {
        // TODO: Verification of taskIds.
        self.lock().unwrap().mut_user(id).map(|u| {
            u.easiness = o;
            u.easiness.clone()
        })
    }
    fn swap_user_importance(&self, id: UserId, from: usize, to: usize) -> Option<OrderedTasks> {
        let mut space = self.lock().unwrap();
        let u = space.mut_user(id)?;
        u.move_importance(from, to);
        Some(u.importance.clone())
    }
    fn swap_user_easiness(&self, id: UserId, from: usize, to: usize) -> Option<OrderedTasks> {
        let mut space = self.lock().unwrap();
        let u = space.mut_user(id)?;
        u.move_easiness(from, to);
        Some(u.easiness.clone())
    }
}

fn combine_important_and_easy(importance: &OrderedTasks, easiness: &OrderedTasks) -> OrderedTasks {
    let mut ords = BTreeMap::<TaskId, (usize, usize)>::new();
    for (ord, id) in importance.iter().enumerate() {
        assert!(ords.insert(id, (ord, 0)).is_none());
    }
    for (ord, id) in easiness.iter().enumerate() {
        ords.get_mut(&id).unwrap().1 = ord;
    }
    let mut ords = ords
        .into_iter()
        .map(|(id, (importance, easiness))| (importance + easiness, importance, id))
        .collect::<Vec<_>>();
    ords.sort();
    OrderedTasks(ords.into_iter().map(|(_, _, id)| id).collect())
}

// One implementation of an AthensSpace
#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct SimpleAthensSpace {
    id: SpaceId,
    alias: String,
    tasks: Vec<Task>,
    users: Vec<UserWithOrds>,
}
impl Default for SimpleAthensSpace {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct UserWithOrds {
    user: User,
    importance: OrderedTasks, // TODO: Setter
    easiness: OrderedTasks,
}

impl UserWithOrds {
    pub fn move_importance(&mut self, from: usize, to: usize) -> &mut Self {
        self.importance.reorder(from, to);
        self
    }
    pub fn move_easiness(&mut self, from: usize, to: usize) -> &mut Self {
        self.easiness.reorder(from, to);
        self
    }
}

impl SimpleAthensSpace {
    pub fn new() -> Self {
        Self {
            id: SpaceId(0), // TODO
            alias: "My space".to_string(),
            tasks: vec![],
            users: vec![],
        }
    }
    pub fn new_user(&mut self) -> &mut UserWithOrds {
        let id = UserId(self.users.len());
        let default_order = OrderedTasks(self.task_ids().collect());
        self.users.push(UserWithOrds {
            user: User {
                id,
                alias: String::new(),
                weight: 1,
            },
            importance: default_order.clone(),
            easiness: default_order,
        });
        self.users.last_mut().unwrap()
    }
    pub fn task_ids<'a>(&'a self) -> impl Iterator<Item=TaskId> + 'a {
        self.tasks.iter().map(|t| t.id)
    }
    pub fn user(&self, id: UserId) -> &UserWithOrds {
        &self.users[id.0]
    }
    pub fn mut_user(&mut self, id: UserId) -> Option<&mut UserWithOrds> {
        self.users.get_mut(id.0)
    }
    pub fn new_task(&mut self) -> &mut Task {
        let id = TaskId(self.tasks.len());
        self.tasks.push(Task {
            id,
            text: String::new(),
        });
        for user in self.users.iter_mut() {
            user.importance.push_front(id);
            user.easiness.push_front(id);
        }
        self.tasks.last_mut().unwrap()
    }
    pub fn mut_task(&mut self, id: TaskId) -> Option<&mut Task> {
        self.tasks.get_mut(id.0)
    }
    pub fn task(&self, id: TaskId) -> &Task {
        &self.tasks[id.0]
    }
    pub fn easiness(&self) -> OrderedTasks {
        if self.users.is_empty() {
            return OrderedTasks(self.task_ids().collect());
        }
        let mut ords = Vec::new();
        for u in self.users.iter() {
            // TODO: Unnecessary clone.
            ords.push(u.easiness.clone());
        }
        ranked_pairs_ordering(&ords)
    }
    pub fn importance(&self) -> OrderedTasks {
        if self.users.is_empty() {
            return OrderedTasks(self.task_ids().collect());
        }
        let mut ords = Vec::new();
        for u in self.users.iter() {
            // TODO: Unnecessary clone.
            ords.push(u.importance.clone());
        }
        ranked_pairs_ordering(&ords)
    }
}
