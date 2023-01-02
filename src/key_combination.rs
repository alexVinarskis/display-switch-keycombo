//
// Copyright © 2022 Alex Vinarskis
// This code is licensed under MIT license (see LICENSE.txt for details)
//

use serde::{Deserialize, Deserializer};
use std::fmt;
use rdev::Key;
use crate::key::key_from_string;

#[derive(Clone)]
#[derive(PartialEq)]
pub enum KeyCombination {
    Symbolic(Vec<Key>),
}

impl KeyCombination {
    #[cfg(test)] 
    pub fn new(vec: Vec<Key>) -> Self {
        Self::Symbolic(vec)
    }
    pub fn is_match(&self, vec: &Vec<Key>) -> bool {
        match self {
            Self::Symbolic(keys) => keys.iter().all(|k| vec.contains(k)),
        }
    }
}

impl<'de> Deserialize<'de> for KeyCombination {
    fn deserialize<D>(deserializer: D) -> Result<KeyCombination, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        let strings: Vec<&str> = s.split_whitespace().collect();
        let mut keys: Vec<Key> = Vec::new();
        for string in &strings {
            let key: Key = key_from_string(&string);
            assert!(key != Key::Unknown(99), "Importing .config failed; Key {} did not match existing keys", string);
            keys.push(key)
        }
        Ok(Self::Symbolic(keys))
    }
}

impl fmt::Display for KeyCombination {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Symbolic(sym) => write!(f, "{:?}", sym),
        }
    }
}

impl fmt::Debug for KeyCombination {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Symbolic(keys) => {
                let mut s = String::new();
                for k in keys {
                    s.push_str(&format!("{:?}+", k));
                }
                s.pop();
                write!(f, "{}", s)
            }
        }
    }
}
