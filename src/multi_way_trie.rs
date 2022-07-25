use std::borrow::Borrow;
use std::cmp::{Ordering, PartialOrd};
use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::Sub;
use std::rc::Rc;
use std::time::Instant;

use chrono::Duration;
use concat_string::concat_string;

struct CacheEntry<T> {
    timestamp: Instant,
    entry: T,
}

impl<T> CacheEntry<T> {
    pub fn new(entry: T) -> CacheEntry<T> {
        CacheEntry {
            timestamp: Instant::now(),
            entry,
        }
    }

    pub fn get(&self) -> &T {
        &self.entry
    }

    pub fn get_timestamp(&self) -> &Instant {
        return &self.timestamp;
    }
}

struct CachedFn<P: Eq + Hash, V: Clone> {
    function: Box<dyn FnMut(&P) -> V>,
    invalid_time: Instant,
    cache: HashMap<P, CacheEntry<V>>,
}

impl<P: Eq + Hash, V: Clone> CachedFn<P, V> {
    pub fn new(function: Box<dyn FnMut(&P) -> V>) -> CachedFn<P, V> {
        CachedFn {
            function,
            invalid_time: Instant::now(),
            cache: HashMap::new(),
        }
    }

    pub fn invalidate(&mut self) {
        self.invalid_time = Instant::now();
    }

    pub fn call(&mut self, param: P) -> V {
        if let Some(mut cache_entry) = self.cache.get(&param) {
            if cache_entry.timestamp >= self.invalid_time {
                return (*cache_entry).get().clone();
            }
        }

        let call_res = (self.function)(&param);
        self.cache.insert(param, CacheEntry::new(call_res.clone()));
        call_res
    }
}

pub struct MultiWayTrie {
    root: Rc<MWTNode>,
    size: usize,
    suggestion_cache: CachedFn<String, Option<String>>,
}

impl MultiWayTrie {
    pub fn new() -> MultiWayTrie {
        let root = Rc::new(MWTNode::new());
        let root_clone = root.clone();
        let find_suggestion = move |partial_word: &String| {
            let in_word = partial_word.chars();

            let mut suggestion: Vec<char> = vec![];
            let mut current_node: &MWTNode = root_clone.borrow();
            while !current_node.is_word {
                if let Some(child) = current_node.get_youngest_child() {
                    suggestion.push(*child.0);
                    current_node = &child.1.borrow();
                }
                else {
                    return None;
                }
            }

            Some(String::from_iter(suggestion.iter()))
        };
        MultiWayTrie {
            root,
            size: 0,
            suggestion_cache: CachedFn::new(Box::new(find_suggestion)),
        }
    }

    pub fn get_time_based_suggestion(&mut self, partial_word: String) -> Option<String> {
        self.suggestion_cache.call(partial_word)
    }
}

#[derive(PartialOrd, PartialEq)]
struct MWTNode {
    timestamp: Instant,

    children: BTreeMap<char, Box<MWTNode>>,

    is_word: bool,
}

impl MWTNode {
    pub fn new() -> MWTNode {
        MWTNode {
            is_word: false,
            children: BTreeMap::new(),
            timestamp: Instant::now(),
        }
    }

    pub fn get_youngest_child(&self) -> Option<(&char, &Box<MWTNode>)> {
        self.children.iter()
            .min_by(|(_, node1), (_, node2)|
                        node1.timestamp.cmp(&node2.timestamp)
            )
    }

    pub fn get_child_for(&mut self, next: char) -> Option<&MWTNode> {
        match self.children.get(&next) {
            Some(child_box) => Some(child_box),
            None => None
        }
    }

    pub fn insert(&mut self, word: &str) {
        if let Some(c) = word.chars().next() {
            let next_str = &word[1..];
            if let Some(child) = self.children.get_mut(&c) {
                child.insert(next_str);
            } else {
                let mut node = MWTNode::new();
                node.insert(next_str);
                self.children.insert(c, Box::new(node));
            }
        } else {
            self.is_word = true
        }
    }
}