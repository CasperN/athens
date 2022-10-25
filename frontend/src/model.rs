use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
struct EntryData(String);

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Model {
    entries: Vec<EntryData>,
    // NOTE: A doubly linked list would be more efficient for random insert/remove
    importance: Vec<usize>,
    easiness: Vec<usize>,
}
impl Model {
    pub fn add_entry(&mut self) {
        assert_eq!(self.entries.len(), self.importance.len());
        assert_eq!(self.entries.len(), self.easiness.len());

        let i = self.entries.len();
        self.entries.push(EntryData(String::new()));
        self.importance.insert(0, i);
        self.easiness.insert(0, i);
    }
    pub fn move_importance(&mut self, from: usize, to: usize) {
        if from == to {
            return;
        }
        let i = self.importance.remove(from);
        self.importance.insert(to, i);
    }

    pub fn move_easiness(&mut self, from: usize, to: usize) {
        // TODO: deduplicate with move_importance
        if from == to {
            return;
        }
        let i = self.easiness.remove(from);
        self.easiness.insert(to, i);
    }

    pub fn set_text(&mut self, id: usize, text: String) {
        self.entries[id].0 = text;
    }
    // TODO: Yield a real type with names
    // TODO: Should this output a reference?
    pub fn iter_importance(&self) -> Vec<(usize, usize, String)> {
        self.importance
            .iter()
            .enumerate()
            .map(|(order, &id)| (order, id, self.entries[id].0.clone()))
            .collect()
    }
    pub fn iter_easiness(&self) -> Vec<(usize, usize, String)> {
        self.easiness
            .iter()
            .enumerate()
            .map(|(order, &id)| (order, id, self.entries[id].0.clone()))
            .collect()
    }

}
