
use std::mem;

#[derive(Debug, Clone)]
pub struct VEBTree {
    // box is necessary for recursion
    children: Vec<Option<VEBTree>>,
    summary: Option<Box<VEBTree>>,
    // special cases of min and max:
    // if the tree is empty, min > max
    // if the tree contains only one element, min == max == that element.
    min: i64,
    max: i64,
    universe: i64,
    sqrt_universe: i64,
}

// helper macros

macro_rules! subtree {
    ( $self_: ident, $x: expr ) => {
        $self_.children.get($x).expect("child idx out of bounds").as_ref()
    }
}

macro_rules! summary {
    ( $self_: ident ) => {
        $self_.summary.as_ref().expect("summary not present")
    }
}

macro_rules! summary_mut {
    ( $self_: ident ) => {
        $self_.summary.as_mut().expect("summary not present")
    }
}

impl VEBTree {
    fn high(&self, x: i64) -> i64 {
        ((x as f64) / (self.sqrt_universe as f64)).floor() as i64
    }

    fn low(&self, x: i64) -> i64 {
        x % self.sqrt_universe
    }

    fn index(&self, i: i64, j: i64) -> i64 {
        i * self.sqrt_universe + j
    }

    pub fn new(max_elem: i64) -> Result<Self, &'static str> {
        if max_elem <= 1 {
            Err("universe size must be > 1")
        } else if max_elem > isize::max_value() as i64 {
            Err("universe too big")
        } else {
            // sqrt_universe: 2^(floor(log_2(universe) / 2))
            // let sqrt_universe = 2i64.pow(((max_elem as f64).ln() / (2f64.ln()) / 2f64).floor() as u32);
            let sqrt_universe = (((max_elem as f64).ln() / (2f64).ln()) / 2f64).exp2() as i64;
            Ok(VEBTree {
                universe: max_elem,
                sqrt_universe: sqrt_universe,
                min: max_elem,
                max: -1,
                summary: if max_elem == 2 {
                    None
                } else {
                    Some(Box::new(VEBTree::new(sqrt_universe).unwrap()))
                },
                children: if max_elem == 2 {
                    vec![]
                } else {
                    vec![None; sqrt_universe as usize]
                },
            })
        }
    }

    // =========
    // observers
    // =========

    pub fn minimum(&self) -> i64 {
        self.min
    }

    pub fn maximum(&self) -> i64 {
        self.max
    }

    pub fn universe(&self) -> i64 {
        self.universe
    }

    pub fn is_empty(&self) -> bool {
        self.min > self.max
    }

    pub fn has(&self, x: i64) -> bool {
        if x == self.min || x == self.max {
            true
        } else if self.universe == 2 || x > self.universe {
            false
        } else {
            subtree!(self, self.high(x) as usize).map_or(false, |subtree| {
                subtree.has(self.low(x))
            })
        }
    }

    pub fn find_next(&self, x: i64) -> Option<i64> {
        // base case
        if self.is_empty() {
            None
        } else if self.universe == 2 {
            if x == 0 && self.max == 1 {
                Some(1)
            } else {
                None
            }
        } else if x < self.min {
            Some(self.min)
        } else {
            let idx = self.high(x);
            let low = self.low(x);
            // look in subtrees
            subtree!(self, idx as usize).map_or_else(|| {
                self.find_subtree(x)
            }, |subtree| {
                let max_low = subtree!(self, idx as usize).unwrap().maximum();
                if low < max_low {
                    Some(self.index(idx, subtree.find_next(low).unwrap()))
                } else {
                    self.find_subtree(x)
                }
            })
        }
    }

    fn find_subtree(&self, x: i64) -> Option<i64> {
        // subtree not present - we need to look in a different cluster. Since universe > 2, we know summary exists.
        summary!(self).find_next(self.high(x)).map_or(None, |next_index| {
            Some(self.index(next_index, subtree!(self, next_index as usize).unwrap().minimum()))
        })
    }

    // ========
    // mutators
    // ========

    fn empty_insert(&mut self, x: i64) {
        self.min = x;
        self.max = x;
    }

    pub fn insert(&mut self, mut x: i64) {
        if self.is_empty() {
            self.empty_insert(x);
        } else {
            if self.min == self.max {
                if x < self.min {
                    self.min = x;
                }
                if x > self.max {
                    self.max = x;
                }
            }
            if x < self.min {
                mem::swap(&mut self.min, &mut x);
            }
            if x > self.max {
                self.max = x;
            }
            let idx = self.high(x);
            let low = self.low(x);
            let sqrt = self.sqrt_universe;
            let subtree = &mut self.children[idx as usize];
            match *subtree {
                Some(ref mut subtree) => subtree.insert(low),
                None => {
                    let mut new_tree = VEBTree::new(sqrt).unwrap();
                    new_tree.empty_insert(low);
                    mem::replace(subtree, Some(new_tree));
                    summary_mut!(self).insert(idx);
                },
            };
        };
    }

    pub fn delete(&mut self, mut x: i64) {
        if self.min == self.max && self.min == x {
            self.min = self.universe; self.max = -1;
        } else {
            if self.min == x {
                // we need to calculate the new minimum
                self.min = if summary!(self).is_empty() {
                    self.max // return
                } else {
                    // we need to insert the old minimum
                    x = subtree!(self, summary!(self).minimum() as usize).unwrap().minimum();
                    x
                }
            }
            if self.max == x {
                // we need to calculate the new maximum
                self.max = if summary!(self).is_empty() { // only 1 element in the tree
                    self.min
                } else {
                    subtree!(self, summary!(self).maximum() as usize).unwrap().maximum()
                }
            }
            if !summary!(self).is_empty() {
                // recurse
                let idx = self.high(x);
                let low = self.low(x);
                let mut subtree = &mut self.children[idx as usize];
                subtree.as_mut().unwrap().delete(low);
                // don't store empty trees, and remove from summary as well
                if subtree.as_ref().unwrap().is_empty() {
                    subtree.take();
                    summary_mut!(self).delete(idx);
                };
            };
        };
    }

}

#[test]
fn creation() {
    assert!(VEBTree::new(50).is_ok());
}

#[test]
fn creation_fail() {
    assert!(VEBTree::new(1).is_err());
}

#[test]
fn insertion_and_has() {
    let mut tree = VEBTree::new(50).unwrap();
    assert!(!tree.has(25));
    assert!(!tree.has(26));
    tree.insert(25);
    assert!(tree.has(25));
    assert!(!tree.has(26));
    tree.insert(26);
    assert!(tree.has(25));
    assert!(tree.has(26));
}

#[test]
fn is_empty() {
    let mut tree = VEBTree::new(50).unwrap();
    assert!(tree.is_empty());
    tree.insert(25);
    assert!(!tree.is_empty());
    tree.delete(25);
    assert!(tree.is_empty());
}

#[test]
fn find_next() {
    let mut tree = VEBTree::new(50).unwrap();
    println!("find next: empty: {:?}", tree);
    assert!(tree.find_next(0).is_none());
    assert!(tree.find_next(24).is_none());
    assert!(tree.find_next(25).is_none());
    tree.insert(25);
    println!("find next: 25: {:?}", tree);
    assert!(tree.find_next(0).is_some());
    assert!(tree.find_next(24).is_some());
    assert!(tree.find_next(25).is_none());
}

#[test]
fn delete() {
    let mut tree = VEBTree::new(50).unwrap();
    println!("delete: empty: {:?}", tree);
    assert!(!tree.has(25));
    assert!(!tree.has(26));
    tree.insert(25);
    println!("delete: 25: {:?}", tree);
    assert!(tree.has(25));
    assert!(!tree.has(26));
    tree.insert(26);
    println!("delete: 25 and 26: {:?}", tree);
    assert!(tree.has(25));
    assert!(tree.has(26));
    tree.delete(26);
    println!("delete: 26 (1 deletion): {:?}", tree);
    assert!(!tree.has(26));
    assert!(tree.has(25));
    tree.delete(25);
    println!("delete: empty (2 deletions): {:?}", tree);
    assert!(!tree.has(26));
    assert!(!tree.has(25));
}
