use std::collections::HashMap;

pub struct Pool<KeyType, ValueType> {
    items: HashMap<KeyType, Box<ValueType>>,
}

impl<KeyType, ValueType> Pool<KeyType, ValueType>
where
    KeyType: std::cmp::Eq + std::hash::Hash,
{
    pub fn new(reserve_capacity: usize) -> Self {
        let mut items = HashMap::new();
        items.reserve(reserve_capacity);
        Self { items }
    }

    pub fn get(&self, id: &KeyType) -> Option<&ValueType> {
        self.items.get(id).map(|value| value.as_ref())
    }

    pub fn get_mut(&mut self, id: &KeyType) -> Option<&mut ValueType> {
        self.items.get_mut(id).map(|value| value.as_mut())
    }

    pub fn add(&mut self, id: KeyType, item: Box<ValueType>) {
        self.items.insert(id, item);
    }

    pub fn remove(&mut self, id: &KeyType) -> Option<Box<ValueType>> {
        self.items.remove(id)
    }
}
