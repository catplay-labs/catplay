use alloc::string::String;
use core::{
    fmt,
    ops::{Index, IndexMut},
};
use indexmap::IndexMap;
#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer, de, ser::SerializeMap};

use super::Value;

/// Insertion-ordered plist dictionary.
#[derive(Clone, Default, PartialEq)]
pub struct Dictionary {
    map: IndexMap<String, Value, hashbrown::DefaultHashBuilder>,
}

impl Dictionary {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn clear(&mut self) {
        self.map.clear();
    }
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.map.get(key)
    }
    pub fn get_mut(&mut self, key: &str) -> Option<&mut Value> {
        self.map.get_mut(key)
    }
    pub fn contains_key(&self, key: &str) -> bool {
        self.map.contains_key(key)
    }
    pub fn insert(&mut self, key: String, value: Value) -> Option<Value> {
        self.map.insert(key, value)
    }
    pub fn remove(&mut self, key: &str) -> Option<Value> {
        self.map.swap_remove(key)
    }
    pub fn retain<F: FnMut(&String, &mut Value) -> bool>(&mut self, keep: F) {
        self.map.retain(keep);
    }
    pub fn sort_keys(&mut self) {
        self.map.sort_keys();
    }
    pub fn entry<S: Into<String>>(&mut self, key: S) -> indexmap::map::Entry<'_, String, Value> {
        self.map.entry(key.into())
    }
    pub fn len(&self) -> usize {
        self.map.len()
    }
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
    pub fn iter(&self) -> indexmap::map::Iter<'_, String, Value> {
        self.map.iter()
    }
    pub fn iter_mut(&mut self) -> indexmap::map::IterMut<'_, String, Value> {
        self.map.iter_mut()
    }
    pub fn keys(&self) -> indexmap::map::Keys<'_, String, Value> {
        self.map.keys()
    }
    pub fn values(&self) -> indexmap::map::Values<'_, String, Value> {
        self.map.values()
    }
    pub fn values_mut(&mut self) -> indexmap::map::ValuesMut<'_, String, Value> {
        self.map.values_mut()
    }
}

impl fmt::Debug for Dictionary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.map.fmt(f)
    }
}
impl Index<&str> for Dictionary {
    type Output = Value;
    fn index(&self, key: &str) -> &Value {
        &self.map[key]
    }
}
impl IndexMut<&str> for Dictionary {
    fn index_mut(&mut self, key: &str) -> &mut Value {
        self.map.get_mut(key).expect("no entry found for key")
    }
}
impl<K: Into<String>, V: Into<Value>> FromIterator<(K, V)> for Dictionary {
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        Self {
            map: iter
                .into_iter()
                .map(|(key, value)| (key.into(), value.into()))
                .collect(),
        }
    }
}
impl Extend<(String, Value)> for Dictionary {
    fn extend<T: IntoIterator<Item = (String, Value)>>(&mut self, iter: T) {
        self.map.extend(iter);
    }
}
impl IntoIterator for Dictionary {
    type Item = (String, Value);
    type IntoIter = indexmap::map::IntoIter<String, Value>;
    fn into_iter(self) -> Self::IntoIter {
        self.map.into_iter()
    }
}
impl<'a> IntoIterator for &'a Dictionary {
    type Item = (&'a String, &'a Value);
    type IntoIter = indexmap::map::Iter<'a, String, Value>;
    fn into_iter(self) -> Self::IntoIter {
        self.map.iter()
    }
}
impl<'a> IntoIterator for &'a mut Dictionary {
    type Item = (&'a String, &'a mut Value);
    type IntoIter = indexmap::map::IterMut<'a, String, Value>;
    fn into_iter(self) -> Self::IntoIter {
        self.map.iter_mut()
    }
}

#[cfg(feature = "serde")]
impl Serialize for Dictionary {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(self.len()))?;
        for (key, value) in self {
            map.serialize_entry(key, value)?;
        }
        map.end()
    }
}
#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Dictionary {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> de::Visitor<'de> for Visitor {
            type Value = Dictionary;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a plist dictionary")
            }
            fn visit_unit<E: de::Error>(self) -> Result<Dictionary, E> {
                Ok(Dictionary::new())
            }
            fn visit_map<M: de::MapAccess<'de>>(self, mut access: M) -> Result<Dictionary, M::Error> {
                let mut result = Dictionary::new();
                while let Some((key, value)) = access.next_entry()? {
                    result.insert(key, value);
                }
                Ok(result)
            }
        }
        deserializer.deserialize_map(Visitor)
    }
}
