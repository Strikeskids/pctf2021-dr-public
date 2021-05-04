use std::cmp::Ordering;
use std::hash::Hash;
use std::ops::{BitAnd, Range, RangeInclusive};

const MINCHAR: char = '\0';
const MAXCHAR: char = std::char::MAX;

#[derive(Clone, Hash, Debug)]
pub struct CharRange(Range<char>);

impl Ord for CharRange {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.0.start, self.0.end).cmp(&(other.0.start, other.0.end))
    }
}

impl PartialOrd for CharRange {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for CharRange {
    fn eq(&self, other: &Self) -> bool {
        (self.0.start, self.0.end) == (other.0.start, other.0.end)
    }
}

impl Eq for CharRange {}

fn next_char(r: char) -> char {
    std::char::from_u32(r as u32 + 1).unwrap()
}

impl From<char> for CharRange {
    fn from(r: char) -> CharRange {
        CharRange::from(r..=r)
    }
}

impl From<Range<char>> for CharRange {
    fn from(r: Range<char>) -> CharRange {
        CharRange(r)
    }
}

impl From<RangeInclusive<char>> for CharRange {
    fn from(r: RangeInclusive<char>) -> CharRange {
        CharRange(*r.start()..next_char(*r.end()))
    }
}

impl CharRange {
    pub fn at(c: char) -> CharRange {
        CharRange(c..c)
    }

    pub fn start_from(c: char) -> CharRange {
        CharRange(c..MAXCHAR)
    }

    pub fn end_at(c: char) -> CharRange {
        CharRange(MINCHAR..c)
    }

    pub fn contains(&self, ch: &char) -> bool {
        self.0.contains(ch)
    }

    pub const fn all() -> Self {
        Self(MINCHAR..MAXCHAR)
    }

    pub const fn empty() -> Self {
        Self(MINCHAR..MINCHAR)
    }

    pub fn is_empty(&self) -> bool {
        self.0.start >= self.0.end
    }

    pub fn start(&self) -> char {
        self.0.start
    }

    pub fn end(&self) -> char {
        self.0.end
    }

    pub fn intersects(&self, other: &CharRange) -> bool {
        char::max(self.0.start, other.0.start) < char::min(self.0.end, other.0.end)
    }
}

impl BitAnd<&CharRange> for &CharRange {
    type Output = CharRange;

    fn bitand(self, other: &CharRange) -> Self::Output {
        let start = char::max(self.0.start, other.0.start);
        let end = char::max(start, char::min(self.0.end, other.0.end));
        CharRange(start..end)
    }
}

impl From<&CharRange> for (char, char) {
    fn from(range: &CharRange) -> (char, char) {
        (range.0.start, range.0.end)
    }
}

#[test]
fn order_of_ranges() {
    let r1 = CharRange::from('a'..'t');
    let r2 = CharRange::from('f'..MAXCHAR);
    assert!(r1 < r2);
}
