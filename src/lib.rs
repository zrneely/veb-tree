
use std::mem;

#[derive(Debug)]
pub struct VEBTree {
    // box is necessary for recursion
    children: Vec<Option<Box<VEBTree>>>,
    summary: Option<Box<VEBTree>>,
    min: i64,
    max: i64,
    universe: i64,
    sqrt_universe: i64,
}

impl Clone for VEBTree {
    fn clone(&self) -> Self {
        VEBTree {
            children: self.children.clone(),
            summary: self.summary.clone(),
            min: self.min,
            max: self.max,
            universe: self.universe,
            sqrt_universe: self.sqrt_universe,
        }
    }
}

// helper macros

macro_rules! subtree {
    ( $self_: ident, $x: expr ) => {
        $self_.children.get($x).expect("child idx out of bounds").as_ref()
    }
}

macro_rules! subtree_mut {
    ( $self_: ident, $x: expr ) => {
        $self_.children.get_mut($x).expect("child idx out of bounds").as_mut()
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
            Err("universe size must be > 2")
        } else if max_elem > isize::max_value() as i64 {
            Err("universe too big")
        } else {
            // sqrt_universe: 2^(floor(log_2(universe) / 2))
            let sqrt_universe = ((((max_elem as f64).ln()) / (2f64).ln()) / 2f64).exp2() as i64;
            Ok(VEBTree {
                universe: max_elem,
                sqrt_universe: sqrt_universe,
                min: 0 - 1,
                max: 0 - 1,
                summary: if max_elem <= 2 {
                    None
                } else {
                    Some(Box::new(VEBTree::new(sqrt_universe).unwrap()))
                },
                children: if max_elem <= 2 {
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
        println!("has: x: {:?}, high(x): {:?}, length of children: {:?}", x, self.high(x), self.children.len());
        if x == self.max || x == self.min {
            true
        } else if self.universe == 2 || x > self.universe {
            false
        } else {
            subtree!(self, self.high(x) as usize).map_or(false, |subtree| {
                println!("has: recursing! high(x): {:?}, sqrt_universe: {:?}, low(x): {:?}", self.high(x), self.sqrt_universe, self.low(x));
                subtree.has(self.low(x))
            })
        }
    }

    pub fn find_next(&self, x: i64) -> Option<i64> {
        // base case
        if self.universe == 2 {
            if x == 0 && self.max == 1 {
                Some(1)
            } else {
                None
            }
        } else if x < self.min {
            Some(self.min)
        } else {
            // look in subtrees
            subtree!(self, self.high(x) as usize).map_or_else(|| {
                self.find_subtree(x)
            }, |subtree| {
                let max_low = subtree!(self, self.high(x) as usize).unwrap().maximum();
                if self.low(x) < max_low {
                    Some(self.index(self.high(x), subtree.find_next(self.low(x)).unwrap()))
                } else {
                    self.find_subtree(x)
                }
            })
        }
    }

    fn find_subtree(&self, x: i64) -> Option<i64> {
        // subtree not present - we need to look in a different cluster. Since universe > 2, we know summary exists.
        self.summary.as_ref().unwrap().find_next(self.high(x)).map_or(None, |next_index| {
            Some(self.index(next_index, subtree!(self, next_index as usize).unwrap().minimum()))
        })
    }

    // ========
    // mutators
    // ========

    // helper functions for insert

    fn empty_insert(&mut self, x: i64) {
        self.min = x;
        self.max = x;
    }

    pub fn insert(&mut self, mut x: i64) {
        if self.min == -1 {
            self.empty_insert(x);
        } else {
            let universe = self.universe;
            if x < self.min {
                mem::swap(&mut self.min, &mut x);
            }
            if universe > 2 {
                let idx = self.high(x) as usize;
                let low = self.low(x);
                let sqrt = self.sqrt_universe;
                let subtree = &mut self.children[idx];
                match *subtree {
                    Some(ref mut subtree) => subtree.insert(low),
                    None => {
                        let mut new_tree = VEBTree::new(sqrt).unwrap();
                        new_tree.empty_insert(low);
                        mem::replace(subtree, Some(Box::new(new_tree)));
                    },
                };
            }
            if x > self.max {
                self.max = x;
            }
        }
    }

    pub fn delete(&mut self, mut x: i64) {
        // base cases
        if self.min == self.max {
            self.min = -1;
            self.max = -1;
        } else if self.universe == 2 {
            self.min = if x == 0 { 1 } else { 0 };
            self.max = self.min;
        } else {
            if self.min == x {
                self.min = if self.summary.as_ref().unwrap().is_empty() {
                    let first_cluster = self.summary.as_ref().unwrap().minimum();
                    x = self.index(first_cluster, subtree!(self, first_cluster as usize).unwrap().minimum());
                    x
                } else {
                    self.max
                }
            }
            // recurse
            let hi = self.high(x);
            let lo = self.low(x);
            subtree_mut!(self, hi as usize).unwrap().delete(lo);
            self.max = if subtree!(self, hi as usize).unwrap().minimum() == (0 - 1) {
                self.summary.as_mut().unwrap().delete(hi);
                subtree!(self, hi as usize).take();
                if x == self.max {
                    let summary_max = self.summary.as_ref().unwrap().maximum();
                    if summary_max == -1 {
                        self.min
                    } else {
                        self.index(summary_max, subtree!(self, summary_max as usize).unwrap().maximum())
                    }
                } else {
                    self.max
                }
            } else if x == self.max {
                self.index(self.high(x), subtree!(self, hi as usize).unwrap().maximum())
            } else {
                self.max
            }
        }
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
