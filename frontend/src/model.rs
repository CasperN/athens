use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
struct EntryData(String);

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Model {
    entries: Vec<EntryData>,
    // NOTE: A doubly linked list would be more efficient for random insert/remove
    ordering: Vec<usize>,
}
impl Model {
    pub fn add_entry(&mut self) {
        assert_eq!(self.entries.len(), self.ordering.len());
        let i = self.entries.len();
        self.entries.push(EntryData(String::new()));
        self.ordering.insert(0, i);
    }
    pub fn move_entries(&mut self, from: usize, to: usize) {
        if from == to {
            return;
        }
        let i = self.ordering.remove(from);
        self.ordering.insert(to, i);
    }
    pub fn set_text(&mut self, id: usize, text: String) {
        self.entries[id].0 = text;
    }
    // TODO: Yield a real type with names
    // TODO: Should this output a reference?
    pub fn iter_entries(&self) -> Vec<(usize, usize, String)> {
        self.ordering
            .iter()
            .enumerate()
            .map(|(order, &id)| (order, id, self.entries[id].0.clone()))
            .collect()
    }
}
