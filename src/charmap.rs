use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};
use std::iter::FromIterator;
use std::mem;
use std::ops::{BitAnd, BitOr, Bound::*, Range};

use crate::charrange::CharRange;

pub enum InOrOut<In, Out> {
    In(In),
    Out(Out),
}

#[derive(Clone, Debug)]
pub struct Charmap<V> {
    data: BTreeMap<CharRange, V>,
}

impl<V> Charmap<V> {
    pub fn new() -> Self {
        Self {
            data: BTreeMap::new(),
        }
    }

    pub fn last_key(&self) -> Option<&CharRange> {
        self.data.keys().next_back()
    }

    pub fn split_off(&mut self, at: &char) -> (Option<(CharRange, V)>, Self) {
        let right = Self {
            data: self.data.split_off(&CharRange::at(*at)),
        };
        if let Some(last_key) = self.last_key() {
            if last_key.contains(at) {
                let last_key = last_key.clone();
                return (self.data.remove_entry(&last_key), right);
            }
        }
        return (None, right);
    }

    pub fn intersects(&self, key: &CharRange) -> bool {
        let (start, end) = key.into();
        let range_range = CharRange::at(start)..CharRange::at(end);
        self.contains_key(&start) || self.data.range(range_range).any(|_| true)
    }

    pub fn try_insert(&mut self, key: CharRange, value: V) -> bool {
        if self.intersects(&key) {
            false
        } else {
            self.data.insert(key, value);
            true
        }
    }

    pub fn insert(&mut self, key: CharRange, value: V) {
        if self.intersects(&key) {
            let (start, end) = (&key).into();
            let (split_right, mut keep) = self.split_off(&end);
            let (split_left, _) = self.split_off(&start);
            if let Some((k, v)) = split_left {
                self.data.insert(CharRange::from(k.start()..start), v);
            }
            if let Some((k, v)) = split_right {
                self.data.insert(CharRange::from(end..k.end()), v);
            }
            self.data.append(&mut keep.data);
        }
        self.data.insert(key.into(), value);
    }

    pub fn get_in_or_out(&self, ch: &char) -> InOrOut<(&CharRange, &V), CharRange> {
        let below = if let Some((last_smaller, data)) = self
            .data
            .range((Unbounded, Included(CharRange::start_from(*ch))))
            .next_back()
        {
            if last_smaller.contains(ch) {
                return InOrOut::In((last_smaller, data));
            }
            last_smaller.end()
        } else {
            '\0'
        };
        let above = if let Some((first_bigger, _)) = self
            .data
            .range((Excluded(CharRange::at(below)), Unbounded))
            .next()
        {
            first_bigger.start()
        } else {
            std::char::MAX
        };
        InOrOut::Out(CharRange::from(below..above))
    }

    pub fn get_entry(&self, ch: &char) -> Option<(&CharRange, &V)> {
        if let Some((last_smaller, data)) = self
            .data
            .range((Unbounded, Included(CharRange::start_from(*ch))))
            .next_back()
        {
            if last_smaller.contains(ch) {
                return Some((last_smaller, data));
            }
        }
        None
    }

    pub fn get(&self, ch: &char) -> Option<&V> {
        self.get_entry(ch).map(|(_, v)| v)
    }

    pub fn contains_key(&self, ch: &char) -> bool {
        self.get_entry(ch).is_some()
    }

    pub fn range_values(&self) -> std::collections::btree_map::Iter<'_, CharRange, V> {
        self.data.iter()
    }
}

fn insert_and_increment<V>(
    data: &mut BTreeMap<CharRange, V>,
    position: &mut Option<char>,
    key: Range<char>,
    value: V,
) {
    if let Some(p) = position {
        assert!(key.start >= *p)
    };
    *position = Some(key.end);
    let previous = data.insert(key.into(), value);
    assert!(previous.is_none());
}

impl<V: Clone> Charmap<V> {
    pub fn merge<F>(&mut self, right: &mut Charmap<V>, mut combine: F)
    where
        F: FnMut(&Range<char>, &V, &V) -> V,
    {
        let mut liter = mem::take(&mut self.data).into_iter();
        let mut riter = mem::take(&mut right.data).into_iter();
        let mut lelem = liter.next();
        let mut relem = riter.next();
        let mut position = None;
        while lelem.is_some() && relem.is_some() {
            let (lrange, lvalue) = &lelem.as_ref().unwrap();
            let (rrange, rvalue) = &relem.as_ref().unwrap();
            let start = position.unwrap_or(char::min(lrange.start(), rrange.start()));
            let end = char::min(lrange.end(), rrange.end());
            assert!(start < lrange.end() || start < rrange.end());
            if start >= lrange.start() && start >= rrange.start() {
                let range = start..end;
                let value = combine(&range, &lvalue, &rvalue);
                insert_and_increment(&mut self.data, &mut position, range, value);
                if end == lrange.end() {
                    lelem = liter.next();
                } else {
                    assert_eq!(end, rrange.end());
                    relem = riter.next();
                }
            } else if end > lrange.start() && end > rrange.start() {
                let end = char::max(lrange.start(), rrange.start());
                insert_and_increment(
                    &mut self.data,
                    &mut position,
                    start..end,
                    if lrange.start() < end {
                        lvalue.clone()
                    } else {
                        rvalue.clone()
                    },
                );
            } else if end <= rrange.start() {
                assert_eq!(end, lrange.end());
                insert_and_increment(&mut self.data, &mut position, start..end, lelem.unwrap().1);
                lelem = liter.next();
            } else {
                assert!(end <= lrange.start());
                assert_eq!(end, rrange.end());
                insert_and_increment(&mut self.data, &mut position, start..end, relem.unwrap().1);
                relem = riter.next();
            }
        }

        if let Some(first) = lelem.or(relem) {
            for (range, value) in std::iter::once(first).chain(liter).chain(riter) {
                let start = position.unwrap_or(range.start());
                insert_and_increment(&mut self.data, &mut position, start..range.end(), value);
            }
        }
    }
}

impl<T> FromIterator<(CharRange, T)> for Charmap<T> {
    fn from_iter<I: IntoIterator<Item = (CharRange, T)>>(iter: I) -> Self {
        let mut map = Self::new();
        for (key, value) in iter {
            map.insert(key, value);
        }
        map
    }
}

#[derive(Clone, Debug)]
pub struct Charset(Charmap<()>);

pub struct Ranges<'a, V> {
    inner: std::collections::btree_map::Keys<'a, CharRange, V>,
}

impl<'a, V> Iterator for Ranges<'a, V> {
    type Item = &'a CharRange;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

pub struct Holes<'a, V> {
    inner: std::collections::btree_map::Keys<'a, CharRange, V>,
    start: char,
}

impl<'a, V> Iterator for Holes<'a, V> {
    type Item = CharRange;

    fn next(&mut self) -> Option<Self::Item> {
        for range in &mut self.inner {
            if range.start() > self.start {
                let result = CharRange::from(self.start..range.start());
                self.start = range.end();
                return Some(result);
            }
        }
        let last = CharRange::start_from(self.start);
        if !last.is_empty() {
            return Some(last);
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl Charset {
    pub fn all() -> Self {
        Charset(Charmap {
            data: vec![(CharRange::all(), ())].into_iter().collect(),
        })
    }

    pub fn new() -> Self {
        Charset(Charmap::new())
    }

    pub fn contains(&self, ch: &char) -> bool {
        self.0.contains_key(ch)
    }

    pub fn get(&self, ch: &char) -> Option<&CharRange> {
        self.0.get_entry(ch).map(|(k, _)| k)
    }

    pub fn insert_char(&mut self, ch: char) {
        self.0.insert(CharRange::from(ch), ())
    }

    pub fn insert(&mut self, range: CharRange) {
        self.0.insert(range, ())
    }

    pub fn invert(&mut self) {
        let mut start = '\0';
        let iter = mem::take(&mut self.0.data).into_iter();
        for (k, ()) in iter {
            if k.start() > start {
                self.insert(CharRange::from(start..k.start()));
            }
            start = k.end();
        }
        if start < std::char::MAX {
            self.insert(CharRange::start_from(start))
        }
    }

    pub fn get_in_or_out(&self, ch: &char) -> InOrOut<&CharRange, CharRange> {
        use InOrOut::*;
        match self.0.get_in_or_out(ch) {
            In((k, _)) => In(k),
            Out(k) => Out(k),
        }
    }

    pub fn ranges(&self) -> Ranges<'_, ()> {
        Ranges {
            inner: self.0.data.keys(),
        }
    }

    pub fn holes(&self) -> Holes<'_, ()> {
        Holes {
            inner: self.0.data.keys(),
            start: '\0',
        }
    }

    pub fn len(&self) -> usize {
        self.ranges()
            .map(|rng| rng.end() as u32 - rng.start() as u32)
            .sum::<u32>() as usize
    }
}

impl FromIterator<CharRange> for Charset {
    fn from_iter<I: IntoIterator<Item = CharRange>>(iter: I) -> Self {
        let mut set = Self::new();
        for c in iter {
            set.insert(c)
        }
        set
    }
}

impl FromIterator<char> for Charset {
    fn from_iter<I: IntoIterator<Item = char>>(iter: I) -> Self {
        let mut set = Self::new();
        for c in iter {
            set.insert_char(c)
        }
        set
    }
}

impl Ord for Charset {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.data.keys().cmp(other.0.data.keys())
    }
}

impl PartialOrd for Charset {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Charset {
    fn eq(&self, other: &Self) -> bool {
        self.0.data.keys().eq(other.0.data.keys())
    }
}

impl Eq for Charset {}

impl Hash for Charset {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for v in self.0.data.keys() {
            v.hash(state)
        }
    }
}

impl BitAnd<&Charset> for &Charset {
    type Output = Charset;

    fn bitand(self, other: &Charset) -> Self::Output {
        let mut result: Charset = self.holes().chain(other.holes()).collect();
        result.invert();
        result
    }
}

impl BitOr<&Charset> for &Charset {
    type Output = Charset;
    fn bitor(self, other: &Charset) -> Self::Output {
        self.ranges().chain(other.ranges()).cloned().collect()
    }
}

#[test]
fn charset_membership() {
    let set = Charset::all();
    assert!("\0abc0932".chars().all(|c| set.contains(&c)));

    let set: Charset = "abcd".chars().collect();
    assert!("\00932".chars().all(|c| !set.contains(&c)));
    assert!("abcd".chars().all(|c| set.contains(&c)));

    let set: Charset = vec![('a'..'t'), ('B'..'Y')]
        .into_iter()
        .map(CharRange::from)
        .collect();
    assert!("afghBL".chars().all(|c| set.contains(&c)));
    assert!("zZA932".chars().all(|c| !set.contains(&c)));
}
