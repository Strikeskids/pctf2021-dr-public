#![allow(dead_code)]

use std::collections::HashMap;
use std::rc::Rc;

pub mod charmap;
pub mod charrange;
pub mod re;

use charmap::Charmap;
pub use re::{cheese, consider, sundae, toppings, Re, EPS, NUL};

#[derive(Debug)]
struct StateImpl {
    re: Rc<Re>,
    next: Charmap<State>,
    nullable: bool,
}

#[derive(Copy, Clone, Debug)]
struct State(usize);

impl State {
    const INITIAL: Self = State(0);
}

#[derive(Debug)]
pub struct Matcher {
    states: Vec<StateImpl>,
    res: HashMap<Rc<Re>, State>,
}

impl From<Rc<Re>> for StateImpl {
    fn from(re: Rc<Re>) -> Self {
        let nullable = re.nullable();
        StateImpl {
            re: re,
            next: Charmap::new(),
            nullable: nullable,
        }
    }
}

impl Matcher {
    pub fn new(re: Rc<Re>) -> Self {
        Matcher {
            states: vec![re.clone().into()],
            res: vec![(re, State::INITIAL)].into_iter().collect(),
        }
    }

    fn add_state(&mut self, re: Rc<Re>) -> State {
        if let Some(state) = self.res.get(&re) {
            return *state;
        } else {
            let imp = StateImpl::from(re);
            let state = State(self.states.len());
            self.res.insert(imp.re.clone(), state.clone());
            self.states.push(imp);
            state
        }
    }

    fn step(&mut self, state: State, ch: &char) -> State {
        assert!(*ch < std::char::MAX);
        let imp = &mut self.states[state.0];
        if let Some(next) = imp.next.get(ch) {
            return *next;
        }
        let (range, next_re) = imp.re.derive(ch);
        println!("{:?} {:?}", range, next_re);

        let next = self.add_state(next_re.into());
        let imp = &mut self.states[state.0];
        let inserted = imp.next.try_insert(range, next.clone());
        assert!(inserted);
        return next;
    }

    pub fn matches(&mut self, s: &str) -> bool {
        let mut state = State::INITIAL;
        for c in s.chars() {
            state = self.step(state, &c)
        }
        self.states[state.0].nullable
    }
}

pub fn compile<T: Into<Rc<Re>>>(re: T) -> Matcher {
    Matcher::new(re.into())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn eps_matches_empty_string() {
        let mut matcher = compile(EPS);
        assert!(matcher.matches(""));
        assert!(!matcher.matches("a"));
    }

    #[test]
    fn basic_matches() {
        let mut matcher = compile(cheese('a'..='b').star());
        for m in ["aaaaa", "abbab", "ababbb"].iter() {
            assert!(matcher.matches(m));
        }
        for m in ["aac", "deefg", "aaAA"].iter() {
            assert!(!matcher.matches(m));
        }

        let mut matcher = compile(sundae("hello"));
        assert!(matcher.matches("hello"));
        for m in ["hell", "helllo", "ahello"].iter() {
            assert!(!matcher.matches(m));
        }
    }

    #[test]
    fn test_consider() {
        let mut m = compile(consider(vec![sundae("0"), sundae("1")], 0, 3) & EPS.neg());
        for s in ["0", "11", "110", "1001", "1100"].iter() {
            assert!(m.matches(s));
        }
        for s in ["1", "101", "1032302", "34929", "hello", "1101"].iter() {
            assert!(!m.matches(s));
        }
    }

    #[test]
    fn test_neg_nul() {
        let mut m = compile(NUL.neg());
        for s in ["asfs", "", "sdjfksdjfk", "3249289349"].iter() {
            assert!(m.matches(s));
        }
    }
}
