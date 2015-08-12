
pub struct VEBTree {
    // box is necessary for recursion
    children: Vec<Option<Box<VEBTree>>>,
    aux: Option<Box<VEBTree>>,
    min: u64,
    max: u64,
    max_universe: u64,
}

impl VEBTree {
    pub fn new(max_elem: u64) -> Self {
        VEBTree {
            children: Vec::new(),
            aux: None,
            min: 0,
            max: 0,
            max_universe: max_elem,
        }
    }

    pub fn find_next(&self, x: u64) -> Option<u64> {
        if x <= self.min {
            Some(self.min)
        } else if x > self.max {
            None
        } else {
            let i = ((x as f64) / (self.max_universe as f64).sqrt()).floor() as u64;
            let lo = x % (self.max_universe as f64).sqrt() as u64;
            let hi = x - lo;
            let child = self.get_child(i as usize);
            match *child {
                Some(ref subtree) => {
                    if lo <= subtree.max {
                        Some(hi + subtree.find_next(lo).expect("subtree in invalid state"))
                    } else {
                        match self.aux {
                            Some(ref auxtree) => {
                                match *self.get_child(auxtree.find_next(i).expect("auxtree in invalid state") as usize) {
                                    Some(ref other) => Some(hi + other.min),
                                    None => panic!("next subtree not present"),
                                }
                            }
                            None => panic!("auxtree not present"),
                        }
                    }
                },
                None => panic!("subtree not present"),
            }
        }
    }

    pub fn insert(&mut self, mut x: u64) {
        if x > self.max_universe {
            panic!("can not insert element greater than universe size into vEB tree")
        }
        if self.min > self.max {
            self.min = x;
            self.max = x;
            return;
        }
        if self.min == self.max {
            if x < self.min {
                self.min = x;
            }
            if x > self.max {
                self.max = x;
            }
        }
        if x < self.min {
            let tmp = self.min;
            self.min = x;
            x = tmp;
        }
        if x > self.max {
            self.max = x;
        }
        let i = ((x as f64) / (self.max_universe as f64).sqrt()).floor() as u64;
        let child = self.get_child_mut(i as usize);
        match *child {
            Some(ref mut subtree) => {
                subtree.insert(x % (self.max_universe as f64).sqrt() as u64);
                if subtree.min == subtree.max {
                    match self.aux {
                        Some(ref mut auxtree) => auxtree.insert(i),
                        None => panic!("auxtree not present"),
                    }
                }
            }
            None => panic!("subtree not present"),
        }
    }

    fn get_child(&self, index: usize) -> &Option<Box<VEBTree>> {
        match self.children.get(index) {
            Some(ref child) => child,
            None => panic!("index out of bounds in vEB tree"),
        }
    }

    fn get_child_mut(&mut self, index: usize) -> &mut Option<Box<VEBTree>> {
        match self.children.get_mut(index) {
            Some(ref mut child) => child,
            None => panic!("index out of bounds in vEB tree"),
        }
    }
}

#[test]
fn it_works() {
}
