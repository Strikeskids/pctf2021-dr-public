use std::cmp::Ordering;
use std::collections::HashSet;
use std::hash::Hash;
use std::ops::{BitAnd, BitOr, Mul};
use std::rc::Rc;

use delegate::delegate;

use crate::charmap::{Charset, InOrOut::*};
use crate::charrange::CharRange;

#[derive(Eq, Hash, Debug)]
pub struct Res(Vec<Rc<Re>>);

#[derive(Clone, Eq, PartialEq, PartialOrd, Ord, Hash, Debug)]
pub enum Re {
    Nul,
    Eps,
    Chars(Charset),
    Neg(Rc<Re>),
    Alt(Res),
    And(Res),
    Seq(Rc<Re>, Rc<Re>),
    Star(Rc<Re>),
    Fan(Rc<Re>, usize),
    Lit(&'static str),
    Moon(Rc<Re>, usize, usize),
    Consider(Rc<Res>, usize, usize, usize),
}

impl Res {
    delegate! {
        to self.0 {
            pub fn iter(&self) -> std::slice::Iter<'_, Rc<Re>>;
            pub fn len(&self) -> usize;
        }
    }
}

impl<'a> From<&'a Res> for &'a Vec<Rc<Re>> {
    fn from(res: &'a Res) -> Self {
        &res.0
    }
}

impl From<Vec<Rc<Re>>> for Res {
    fn from(vec: Vec<Rc<Re>>) -> Self {
        Res(vec)
    }
}
impl Clone for Res {
    fn clone(&self) -> Self {
        Res(self.0.to_vec())
    }
}

impl Ord for Res {
    fn cmp(&self, other: &Self) -> Ordering {
        self.iter().cmp(other.iter())
    }
}

impl PartialOrd for Res {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Res {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Res {
    fn derive(&self, ch: &char) -> (CharRange, Vec<Rc<Re>>) {
        let (ranges, res): (Vec<_>, Vec<_>) = self.iter().map(|re| re.derive(ch)).unzip();
        (
            ranges
                .into_iter()
                .reduce(|a, b| &a & &b)
                .unwrap_or(CharRange::all()),
            res,
        )
    }
}

impl Re {
    pub fn nullable(self: &Self) -> bool {
        match self {
            Re::Nul => false,
            Re::Eps => true,
            Re::Neg(re) => !re.nullable(),
            Re::Chars(_) => false,
            Re::Alt(res) => res.iter().any(|x| x.nullable()),
            Re::And(res) => res.iter().all(|x| x.nullable()),
            Re::Seq(a, b) => a.nullable() && b.nullable(),
            Re::Star(_) => true,
            Re::Fan(_, _) => false,
            Re::Moon(_, phase, planet) => phase == planet,
            Re::Consider(_, value, target, _) => value == target,
            Re::Lit(s) => s.is_empty(),
        }
    }

    fn alt<T: IntoIterator<Item = Rc<Re>>>(parts: T) -> Rc<Re> {
        let mut all = vec![];
        for x in parts {
            match &*x {
                Re::Alt(res) => all.extend(res.iter().cloned()),
                Re::Neg(y) => match **y {
                    Re::Nul => return x,
                    _ => all.push(x),
                },
                Re::Nul => (),
                _ => all.push(x),
            }
        }
        let mut seen = HashSet::new();
        all.retain(|re| seen.insert(re.clone()));
        match all.len() {
            0 => Rc::from(NUL),
            1 => return all.into_iter().next().expect("len = 1"),
            _ => Rc::from(Re::Alt(Res(all))),
        }
    }

    fn and<T: IntoIterator<Item = Rc<Re>>>(parts: T) -> Rc<Re> {
        let mut all = vec![];
        for x in parts {
            match &*x {
                Re::And(res) => all.extend(res.iter().cloned()),
                Re::Neg(y) => match **y {
                    Re::Nul => (),
                    _ => all.push(x),
                },
                Re::Nul => return x,
                _ => all.push(x),
            }
        }
        let mut seen = HashSet::new();
        all.retain(|re| seen.insert(re.clone()));
        match all.len() {
            0 => Rc::from(NUL.neg()),
            1 => return all.into_iter().next().expect("len = 1"),
            _ => Rc::from(Re::And(Res(all))),
        }
    }

    fn seq<A: Into<Rc<Re>>, B: Into<Rc<Re>>>(a: A, b: B) -> Rc<Re> {
        let a = a.into();
        let b = b.into();
        match &*a {
            Re::Nul => Rc::from(Re::Nul),
            Re::Eps => b,
            Re::Seq(x, y) => Self::seq(x.clone(), Self::seq(y.clone(), b)),
            _ => match &*b {
                Re::Nul => Rc::from(Re::Nul),
                Re::Eps => a,
                _ => Rc::from(Re::Seq(a, b)),
            },
        }
    }

    fn neg_rc<T: Into<Rc<Re>>>(re: T) -> Rc<Re> {
        let re = re.into();
        match &*re {
            Re::Neg(re) => re.clone(),
            _ => Rc::from(Re::Neg(re)),
        }
    }

    pub fn derive(&self, ch: &char) -> (CharRange, Rc<Re>) {
        use crate::Re::*;
        match self {
            Nul | Eps => (CharRange::all(), Rc::from(Nul)),
            Chars(set) => match set.get_in_or_out(ch) {
                In(range) => (range.clone(), Rc::from(Eps)),
                Out(range) => (range, Rc::from(Nul)),
            },
            Lit(s) => {
                if s.is_empty() {
                    return (CharRange::all(), Rc::from(Nul));
                } else {
                    let mut indices = s.char_indices();
                    let (_, first) = indices.next().expect("checked non-empty");
                    if *ch == first {
                        (
                            CharRange::from(*ch),
                            if let Some((pos, _)) = indices.next() {
                                Rc::from(Lit(&s[pos..]))
                            } else {
                                Rc::from(EPS)
                            },
                        )
                    } else if *ch < first {
                        (CharRange::end_at(first), Rc::from(NUL))
                    } else {
                        (
                            CharRange::start_from(
                                std::char::from_u32(first as u32 + 1).expect(
                                    "we can't be at the end of the range because first < ch",
                                ),
                            ),
                            Rc::from(NUL),
                        )
                    }
                }
            }
            Alt(res) => {
                let (range, res) = res.derive(ch);
                (range, Self::alt(res.into_iter()))
            }
            And(res) => {
                let (range, res) = res.derive(ch);
                (range, Self::and(res.into_iter()))
            }
            Neg(x) => {
                let (range, x) = x.derive(ch);
                (range, Self::neg_rc(x))
            }
            Seq(a, b) => {
                let (r0, aprime) = a.derive(ch);
                let aprime_b = Self::seq(aprime, b.clone());
                if a.nullable() {
                    let (r1, bprime) = b.derive(ch);
                    (&r0 & &r1, Self::alt(vec![aprime_b, bprime].into_iter()))
                } else {
                    (r0, aprime_b)
                }
            }
            Star(a) => {
                let (range, aprime) = a.derive(ch);
                (range, Self::seq(aprime, self.clone()))
            }
            Fan(a, count) => {
                let next = if *count > 2usize {
                    Rc::from(Fan(a.clone(), count - 1))
                } else {
                    a.clone()
                };
                let (range, aprime) = a.derive(ch);
                (range, Self::seq(aprime, next))
            }
            Moon(a, phase, planet) => {
                let (range, aprime) = a.derive(ch);
                let next_phase = if phase >= planet {
                    phase - planet + 1
                } else {
                    phase + 1
                };
                (
                    range,
                    Self::seq(aprime, Rc::from(Re::Moon(a.clone(), next_phase, *planet))),
                )
            }
            Consider(choices, value, target, within) => {
                let mut index = 0;
                let (range, derived) = choices.as_ref().derive(ch);
                (
                    range,
                    Self::alt(derived.into_iter().map(|re| {
                        let next_value = (value * choices.len() + index) % within;
                        index += 1;
                        Self::seq(
                            re,
                            Rc::from(Re::Consider(choices.clone(), next_value, *target, *within)),
                        )
                    })),
                )
            }
        }
    }
}

use Re::*;

impl BitAnd<Re> for Re {
    type Output = Re;
    fn bitand(self, other: Re) -> Self::Output {
        match &self {
            Chars(set1) => match &other {
                Chars(set2) => return Chars(set1 & set2),
                _ => (),
            },
            _ => (),
        }
        Re::and(vec![Rc::from(self), Rc::from(other)])
            .as_ref()
            .clone()
    }
}

impl BitOr<Re> for Re {
    type Output = Re;
    fn bitor(self, other: Re) -> Self::Output {
        match &self {
            Chars(set1) => match &other {
                Chars(set2) => return Chars(set1 | set2),
                _ => (),
            },
            _ => (),
        }
        Re::alt(vec![Rc::from(self), Rc::from(other)])
            .as_ref()
            .clone()
    }
}

impl Mul<Re> for Re {
    type Output = Re;
    fn mul(self, other: Re) -> Re {
        Re::seq(self, other).as_ref().clone()
    }
}

pub const NUL: Re = Re::Nul;
pub const EPS: Re = Re::Eps;

impl<T: Into<CharRange>> From<T> for Re {
    fn from(v: T) -> Self {
        Re::Chars(vec![v.into()].into_iter().collect())
    }
}

pub fn sundae(s: &'static str) -> Re {
    Re::from(Lit(s))
}

pub fn cheese<T: Into<CharRange>>(rng: T) -> Re {
    Re::from(rng.into())
}

pub fn toppings(chars: &str) -> Re {
    Re::Chars(chars.chars().collect())
}

impl Re {
    pub fn star(self) -> Self {
        Re::Star(Rc::from(self))
    }

    pub fn sun(self) -> Self {
        return self.star() & EPS.neg();
    }

    pub fn fickle(self) -> Self {
        return self | EPS;
    }

    pub fn neg(self) -> Self {
        Re::Neg(Rc::from(self))
    }

    pub fn moon_phase(self, phase: usize, planet: usize) -> Self {
        Re::Moon(Rc::from(self), phase, planet)
    }

    pub fn moon(self, planet: usize) -> Self {
        self.moon_phase(1, planet)
    }

    pub fn fan(self, count: usize) -> Self {
        if count == 0 {
            EPS
        } else if count == 1 {
            self
        } else {
            Fan(Rc::from(self), count)
        }
    }
}

pub fn consider<I: IntoIterator<Item = Re>>(re: I, target: usize, within: usize) -> Re {
    Re::Consider(
        Rc::from(Res(re.into_iter().map(Rc::from).collect())),
        0usize,
        target,
        within,
    )
}
