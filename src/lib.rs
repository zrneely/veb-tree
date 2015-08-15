
use std::mem;

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

macro_rules! subtree {
    ( $self_: ident, $x: expr ) => {
        $self_.children.get($x).unwrap().as_ref()
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
                    vec![None]
                } else {
                    vec![None; sqrt_universe as usize]
                },
            })
        }
    }

    pub fn minimum(&self) -> i64 {
        self.min
    }

    pub fn maximum(&self) -> i64 {
        self.max
    }

    pub fn universe(&self) -> i64 {
        self.universe
    }

    pub fn has(&self, x: i64) -> bool {
        if x == self.max || x == self.min {
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
                self.insert_subtree(x)
            }, |subtree| {
                let max_low = subtree!(self, self.high(x) as usize).unwrap().maximum();
                if self.low(x) < max_low {
                    Some(self.index(self.high(x), subtree.find_next(self.low(x)).unwrap()))
                } else {
                    self.insert_subtree(x)
                }
            })
        }
    }

    fn insert_subtree(&self, x: i64) -> Option<i64> {
        // subtree not present - we need to look in a different cluster. Since universe > 2, we know summary exists.
        self.summary.as_ref().unwrap().find_next(self.high(x)).map_or(None, |next_index| {
            Some(self.index(next_index, subtree!(self, next_index as usize).unwrap().minimum()))
        })
    }

    fn empty_insert(&mut self, x: i64) {
        self.min = x;
        self.max = x;
    }

    pub fn insert(&mut self, x_: i64) {
        if self.min == -1 {
            self.empty_insert(x_);
        } else {
            let mut x = x_;
            let universe = self.universe;
            if x < self.min {
                let tmp = self.min;
                self.min = x;
                x = tmp;
            }
            if universe > 2 {
                let idx = self.high(x) as usize;
                let low = self.low(x);
                let sqrt = self.sqrt_universe;
                let mut subtree = self.children.get_mut(idx).unwrap();
                subtree.map_or_else(|| {
                    let mut new_tree = VEBTree::new(sqrt).unwrap();
                    new_tree.empty_insert(low);
                    mem::replace(subtree, Some(Box::new(new_tree)));
                }, |subtree| {
                    subtree.insert(low);
                });
            }
            if x > self.max {
                self.max = x;
            }
        }
    }

}

#[test]
fn test_cretion() {
    assert!(VEBTree::new(50).is_ok());
}

#[test]
fn test_creation_fail() {
    assert!(VEBTree::new(1).is_err());
}
