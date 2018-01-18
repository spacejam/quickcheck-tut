struct Node<K, V>
where
    K: Ord,
{
    key: K,
    value: V,
    left: Option<Box<Node<K, V>>>,
    right: Option<Box<Node<K, V>>>,
}

#[derive(Default)]
pub struct Tree<K, V>
where
    K: Ord,
{
    root: Option<Node<K, V>>,
}

impl<K, V> Tree<K, V>
where
    K: Ord,
{
    pub fn insert(&mut self, key: K, value: V) {}
    pub fn get(&self, key: &K) -> Option<&V> {
        None
    }
}
