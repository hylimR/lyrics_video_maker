use std::collections::HashMap;

struct Cache {
    map: HashMap<u32, String>,
}

impl Cache {
    fn new() -> Self {
        Self { map: HashMap::new() }
    }

    fn generate(&mut self, key: u32) -> String {
        // Simulate side effect or just computation
        format!("Value: {}", key)
    }

    // Pattern 1: Double Lookup (Current Code)
    fn get_double_lookup(&mut self, key: u32) -> Option<&String> {
        if self.map.contains_key(&key) {
            return self.map.get(&key);
        }

        let val = self.generate(key);
        self.map.insert(key, val);
        self.map.get(&key)
    }

    // Pattern 2: Single Lookup (Proposed)
    fn get_single_lookup(&mut self, key: u32) -> Option<&String> {
        // With NLL, this should work?
        if let Some(val) = self.map.get(&key) {
             return Some(val);
        }
        // If we are here, the borrow from get() should have ended?
        // But the return type binds it to self?

        // Let's see if rustc accepts this.
        let val = self.generate(key);
        self.map.insert(key, val);
        self.map.get(&key)
    }
}

fn main() {
    let mut cache = Cache::new();

    // Warm up
    cache.get_single_lookup(1);

    println!("Value: {:?}", cache.get_single_lookup(1));
    println!("Value: {:?}", cache.get_single_lookup(2));
}
