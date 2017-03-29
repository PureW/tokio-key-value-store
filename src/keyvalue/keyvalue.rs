/// Simple key-value store
///
///

use std::collections::HashMap;


pub struct KeyValueStore {
    kvdata: HashMap<String, String>,
}

impl<'a> KeyValueStore {
    pub fn new() -> KeyValueStore {
        KeyValueStore { kvdata: HashMap::new() }
    }

    pub fn set(&mut self, key: String, val: String) {
        self.kvdata.insert(key, val);
    }
    pub fn get(&'a self, key: &String) -> Option<&'a String> {
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

        let mut kv = KeyValueStore::new();
        for &(ref key, ref value) in tests.iter() {
            kv.set(key.clone(), value.clone());
            assert_eq!(kv.get(&key).unwrap(), value);
        }
    }
}
