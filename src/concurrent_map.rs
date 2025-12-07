// SPDX-License-Identifier: GPL-3.0-only
// 🄯 2021, Alexey Parfenov <zxed@alkatrazstudio.net>

use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, RwLock};

type ConcurrentMapOptValue<V> = Option<V>;
type ConcurrentMapOptValueLock<V> = RwLock<ConcurrentMapOptValue<V>>;
type ConcurrentMapSafeLockedOptValue<V> = Arc<ConcurrentMapOptValueLock<V>>;
type ConcurrentMapSafeHash<K, V> = HashMap<K, ConcurrentMapSafeLockedOptValue<V>>;
type ConcurrentMapLockedSafeHash<K, V> = RwLock<ConcurrentMapSafeHash<K, V>>;

pub struct ConcurrentMap<K, V>(Arc<ConcurrentMapLockedSafeHash<K, V>>);

impl<K: Clone + Eq + Hash, V: Clone> ConcurrentMap<K, V> {
    pub fn new() -> Self {
        return Self(Arc::new(RwLock::new(HashMap::new())));
    }

    fn get_from_map(map: &HashMap<K, Arc<RwLock<Option<V>>>>, key: &K) -> Option<V> {
        if let Some(res) = map.get(key)
            && let Ok(res_guard) = res.read()
            && let Some(s) = &*res_guard
        {
            return Some(s.clone());
        }
        return None;
    }

    pub fn get(&self, key: &K) -> Option<V> {
        if let Ok(guard) = self.0.read() {
            let map = &*guard;
            return Self::get_from_map(map, key);
        }
        return None;
    }

    pub fn set<F>(&self, key: &K, val_func: F) -> Option<V>
    where
        F: FnOnce() -> V,
    {
        let i = Arc::new(RwLock::new(None));

        let mut res_lock = None;
        if let Ok(mut lock) = self.0.write() {
            let map = &mut *lock;
            if let Some(res) = Self::get_from_map(map, key) {
                return Some(res);
            }
            if let Ok(lock) = i.write() {
                res_lock = Some(lock);
                map.insert(key.clone(), i.clone());
            }
        }

        if let Some(mut lock_guard) = res_lock {
            let res = val_func();
            let ret = Some(res.clone());
            *lock_guard = Some(res);
            return ret;
        }

        drop(res_lock);
        panic!();
    }

    pub fn set_if_not_exists<F>(&self, key: &K, val_func: F) -> Option<V>
    where
        F: FnOnce() -> V,
    {
        if let Some(res) = self.get(key) {
            return Some(res);
        }
        return self.set(key, val_func);
    }
}

impl<K: Eq + Hash, V: Clone> Clone for ConcurrentMap<K, V> {
    fn clone(&self) -> Self {
        return Self(self.0.clone());
    }
}
