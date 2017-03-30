/// Simple, concurrent key-value store
///
///

extern crate concurrent_kv;

use std::sync::Arc;

#[derive(Default)]
pub struct KeyValueStore {
    kvdata: concurrent_kv::Library<String, String>,
}

impl<'a> KeyValueStore {
    pub fn new() -> KeyValueStore {
        Self::default()
    }

    pub fn set(&self, key: String, val: String) {
        self.kvdata.insert(key, val);
    }
    pub fn get(&self, key: &String) -> Option<Arc<String>> {
        self.kvdata.get(key)
    }
}

#[cfg(test)]
mod tests {


    use super::KeyValueStore;

    #[test]
    fn test_set_get() {
        let tests = [("foo".to_string(), "bar".to_string()),
                     ("cake".to_string(), "strawberry".to_string())];

        let kv = KeyValueStore::new();
        for &(ref key, ref value) in tests.into_iter() {
            kv.set(key.clone(), value.clone());
            assert_eq!((*kv.get(&key).unwrap()).clone(), value.to_string());
        }
    }
}
