use crate::{OrderedTasks, TaskId};
use std::collections::{BTreeMap, BTreeSet};

// TODO: Weighting.
pub fn ranked_pairs_ordering(orderings: &[OrderedTasks]) -> OrderedTasks {
    if orderings.is_empty() {
        return OrderedTasks(vec![]);
    }
    // TODO: Check that the orderings all have the same number of TaskIds.

    // 1. For each user make a map from TaskId to the user's task order.
    let mut user_to_taskid_to_order = Vec::<BTreeMap<TaskId, usize>>::new();
    for user_ordering in orderings.iter() {
        user_to_taskid_to_order.push(Default::default());
        let task_id_to_order = user_to_taskid_to_order.last_mut().unwrap();
        for (ord, id) in user_ordering.0.iter().enumerate() {
            let prev = task_id_to_order.insert(*id, ord);
            debug_assert!(prev.is_none()); // Ids appear exactly once.
        }
    }
    // 2. For each pair of task ids, compute a 1v1 election win margin.
    let mut win_margins = BTreeMap::<(TaskId, TaskId), i64>::new();
    for (k, &i) in orderings[0].0.iter().enumerate() {
        for &j in orderings[0].0[0..k].iter() {
            // Consider the TaskIds i and j in the lower triangle.
            // WLOG i < j.
            debug_assert_ne!(i, j);
            let (i, j) = if i < j { (i, j) } else { (j, i) };
            for u in user_to_taskid_to_order.iter() {
                let rate = win_margins.entry((i, j)).or_insert(0);
                let i_rank = u.get(&i).unwrap();
                let j_rank = u.get(&j).unwrap();
                if i_rank < j_rank {
                    *rate += 1;
                } else {
                    *rate -= 1;
                }
            }
        }
    }
    // 3. Sort each 1v1 by the win margin.
    let mut win_margins: Vec<(i64, (TaskId, TaskId))> = win_margins
        .into_iter()
        .map(|((i, j), margin)| {
            if margin > 0 {
                (margin, (j, i)) // j > i
            } else {
                (-margin, (i, j)) // i > j
            }
        })
        .collect();
    win_margins.sort();
    // 4. Commit the wins into a directed acyclic graph in order of margin.
    // If an edge creates a cycle, it will not be inserted.
    let mut dag = TaskIdDag::default();
    for (_, edge) in win_margins {
        dag.try_insert(edge);
    }
    // 5. Compute the topological sort to get the final ordering.
    let mut ord = dag.topological_sort();
    ord.reverse(); // TODO: Fix topological_sort so we don't need to reverse.
    OrderedTasks(ord)
}

// Locking in ranked pairs
// Given a list of edges, lock in the edge if it doesn't cause a cycle
// and return the list of nodes in topological ordering
#[derive(Debug, Default)]
struct TaskIdDag(BTreeMap<TaskId, Vec<TaskId>>);
impl TaskIdDag {
    // Inserts the edge so long as it does not introduce a cycle.
    fn try_insert(&mut self, edge: (TaskId, TaskId)) -> bool {
        let (a, b) = edge;
        let mut seen = BTreeSet::<TaskId>::new();
        seen.insert(a);
        // Depth first traversal
        enum ToDo {
            Pop(TaskId),
            Explore(TaskId),
        }
        let mut todo = Vec::<ToDo>::new();
        todo.push(ToDo::Explore(b));
        while let Some(task) = todo.pop() {
            match task {
                ToDo::Pop(id) => {
                    seen.remove(&id);
                }
                ToDo::Explore(id) => {
                    if !seen.insert(id) {
                        return false; // Cycle detected.
                    }
                    todo.push(ToDo::Pop(id));
                    for next_id in self.0.entry(id).or_default().iter() {
                        todo.push(ToDo::Explore(*next_id));
                    }
                }
            }
        }
        self.0.entry(a).or_default().push(b);
        true
    }
    fn topological_sort(&self) -> Vec<TaskId> {
        // TODO: sort needs O(1) lookups too.
        let mut sort = Vec::new();
        while sort.len() < self.0.len() {
            // Consider the nodes not yet in the topological sort.
            let mut unsorted_nodes = self
                .0
                .iter()
                .flat_map(|(node, _)| {
                    if sort.contains(node) {
                        None
                    } else {
                        Some(*node)
                    }
                })
                .collect::<BTreeSet<_>>();

            for (node, children) in self.0.iter() {
                if sort.contains(node) {
                    continue;
                }
                for child in children {
                    unsorted_nodes.remove(child);
                }
            }
            let prev_len = sort.len();
            sort.extend(unsorted_nodes.into_iter());
            assert!(prev_len < sort.len());
        }
        sort
    }
}

#[cfg(test)]
mod test {
    use super::*;
    fn edge(a: usize, b: usize) -> (TaskId, TaskId) {
        (TaskId(a), TaskId(b))
    }
    #[test]
    fn try_insert_succeeds_with_no_cycle() {
        let mut d = TaskIdDag::default();
        assert!(d.try_insert(edge(1, 2)));
        assert!(d.try_insert(edge(1, 3)));
        assert!(d.try_insert(edge(2, 3)));
        assert!(d.try_insert(edge(3, 4)));
    }
    #[test]
    fn try_insert_detects_self_edge() {
        let mut d = TaskIdDag::default();
        assert!(!d.try_insert(edge(1, 1)));
    }
    #[test]
    fn try_insert_detects_cycle_3() {
        let mut d = TaskIdDag::default();
        assert!(d.try_insert(edge(1, 2)));
        assert!(d.try_insert(edge(2, 3)));
        assert!(!d.try_insert(edge(3, 1)));
    }
    #[test]
    fn topological_sort_works_simple() {
        let mut d = TaskIdDag::default();
        d.try_insert(edge(1, 2));
        d.try_insert(edge(2, 3));
        d.try_insert(edge(3, 4));
        assert_eq!(
            d.topological_sort(),
            vec![TaskId(1), TaskId(2), TaskId(3), TaskId(4)]
        );
    }
    #[test]
    fn topological_sort_works_harder() {
        let mut d = TaskIdDag::default();
        d.try_insert(edge(1, 3)); // 1 -> [3, 5]
        d.try_insert(edge(1, 5));
        d.try_insert(edge(2, 3)); // 2 -> 3 -> 4
        d.try_insert(edge(3, 4));
        d.try_insert(edge(7, 6)); // 7 -> 6
        assert_eq!(
            d.topological_sort(),
            vec![
                // No parents:
                TaskId(1),
                TaskId(2),
                TaskId(7),
                // 1 parent:
                TaskId(3),
                TaskId(5),
                TaskId(6),
                // 2 parents:
                TaskId(4)
            ]
        );
    }
}
