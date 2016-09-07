
#![deny(missing_docs,
        missing_debug_implementations, missing_copy_implementations,
        trivial_casts, trivial_numeric_casts,
        unsafe_code,
        unstable_features,
        unused_import_braces, unused_qualifications)]

#![cfg_attr(feature = "dev", allow(unstable_features))]
#![cfg_attr(feature = "dev", feature(plugin))]
#![cfg_attr(feature = "dev", plugin(clippy))]


//! A simple implementation of van Emde Boas trees. For more information on this unique data
//! structure, see the documentation on the `VEBTree` struct.

use std::iter::Iterator;

/// The van Emde Boas tree structure.
///
/// The van Emde Boas tree, named after its creator, is a data structure which stores numbers from
/// 0 to some upper bound which is specified when the data structure is created. That upper bound
/// is known as the "universe size" of the tree - note that the upper bound is exclusive, so it's
/// not possible to store the value equal to the universe size. Van Emde Boas trees support
/// insertion, deletion, querying the existence of an element, and finding the next element given a
/// member of the Universe (which doesn't necessarily have to be in the tree to begin with).
///
/// Van Emde Boas trees are unique in that most operations on them take asymptotic time of
/// `O(log(log(U))`, where U is the universe size. The time of the operations is _not_ related to
/// the number of elements in the tree, and it scales extremely slowly with maximum Universe Size.
/// For example, a `VEBTree` capable of storing every 64-bit number is only 2 times slower than one
/// capable of storing every 32-bit number.
///
/// Van Emde Boas trees therefore are excellent choices in cases where one needs to store a few
/// elements of a massive Universe, and where finding the next element or iterating over previously
/// inserted elements is a useful operation.
#[derive(Debug, Clone)]
pub struct VEBTree {
    // The child trees, stored on the heap to allow an arbitrary number of them and to prevent a
    // recursive struct.
    children: Vec<Option<VEBTree>>,

    // Since Option stores its possible value on the stack, we have to box its contents to keep
    // them on the heap.
    summary: Option<Box<VEBTree>>,

    // The smallest element in the tree, or nothing if the tree is empty. The minimum is not stored
    // in subtrees.
    min: Option<u64>,
    // The largest element in the tree, or nothing if the tree is empty.
    max: Option<u64>,

    // The Universe Size of the tree.
    universe: u64,
    // x % sqrt(universe size) is calculated by taking the lower-order
    // ceil(log2(universe size) / 2) bits. This is a mask to keep only those bits efficiently.
    low_mask: u64,
    // x / sqrt(universe_size) is calculated by  taking the higher-order
    // floor(log2(universe size) / 2) bits. This is a mask and shift amount to calculate this
    // efficiently.
    high_mask: u64,
    high_shift: u64,
    // The universe size of child trees
    sqrt_universe: u64,
}

// helper macros

macro_rules! subtree {
    ( $self_: ident, $x: expr ) => {
        $self_.children.get($x).expect("child index out of bounds").as_ref()
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

    fn high(&self, x: u64) -> u64 {
        (x >> self.high_shift) & self.high_mask
    }

    fn low(&self, x: u64) -> u64 {
        x & self.low_mask
    }

    fn index(&self, i: u64, j: u64) -> u64 {
        (i << self.high_shift) | j
    }

    /// Creates a new van Emde Boas tree.
    ///
    /// # Errors
    ///
    /// Returns an `Err(&'static str)` when the universe size is 0 or 1. Van Emde Boas trees have a
    /// minimum universe size of 2.
    pub fn new(universe: u64) -> Result<Self, &'static str> {
        if universe <= 1 {
            Err("universe size must be > 1")
        } else {
            let shift_amount = ((universe as f64).ln() / 2f64.ln()) / 2f64;
            let sqrt_universe = (universe as f64).sqrt().ceil() as u64;
            Ok(VEBTree {
                universe: universe,
                low_mask: (1 << (shift_amount.ceil() as u64)) - 1,
                high_mask: (1 << (shift_amount.floor() as u64)) - 1,
                high_shift: shift_amount.ceil() as u64,
                sqrt_universe: sqrt_universe,
                min: None,
                max: None,
                summary: if sqrt_universe <= 2 {
                    None
                } else {
                    Some(Box::new(VEBTree::new(sqrt_universe).unwrap()))
                },
                children: vec![None; sqrt_universe as usize],
            })
        }
    }

    /// Returns the lowest value stored in the tree, or None if the tree is empty.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use self::veb_rs::VEBTree;
    /// let mut tree = VEBTree::new(10).unwrap();
    /// assert!(!tree.minimum().is_some());
    ///
    /// tree.insert(4);
    /// tree.insert(6);
    /// assert_eq!(4, tree.minimum().unwrap());
    /// ```
    ///
    /// # Runtime
    /// `O(1)`
    pub fn minimum(&self) -> Option<u64> {
        self.min
    }

    /// Returns the highest value stored in the tree, or None if the tree is empty.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use self::veb_rs::VEBTree;
    /// let mut tree = VEBTree::new(10).unwrap();
    /// assert!(!tree.maximum().is_some());
    ///
    /// tree.insert(4);
    /// tree.insert(6);
    /// assert_eq!(6, tree.maximum().unwrap());
    /// ```
    ///
    /// # Runtime
    /// `O(1)`
    pub fn maximum(&self) -> Option<u64> {
        self.max
    }

    /// Returns the universe size of the tree, which is 1 larger than the largest possible value
    /// the tree can store.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use self::veb_rs::VEBTree;
    /// let mut tree = VEBTree::new(10).unwrap();
    /// assert_eq!(10, tree.universe());
    /// ```
    ///
    /// # Runtime
    /// `O(1)`
    pub fn universe(&self) -> u64 {
        self.universe
    }

    /// Returns true if the tree is empty.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use self::veb_rs::VEBTree;
    /// let mut tree = VEBTree::new(10).unwrap();
    /// assert!(tree.is_empty());
    /// tree.insert(5);
    /// assert!(!tree.is_empty());
    /// ```
    ///
    /// # Runtime
    /// `O(1)`
    pub fn is_empty(&self) -> bool {
        !self.min.is_some()
    }

    /// Returns true if the tree contains the given value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use self::veb_rs::VEBTree;
    /// let mut tree = VEBTree::new(10).unwrap();
    /// tree.insert(5);
    /// assert!(tree.contains(5));
    /// assert!(!tree.contains(6));
    /// ```
    ///
    /// # Runtime
    /// `O(log(log(U)))`
    pub fn contains(&self, x: u64) -> bool {
        if self.is_empty() || self.universe <= 2 || x >= self.universe {
            false
        // Here, since the tree isn't empty, we're guaranteed that both min and max are present.
        } else if x == self.min.unwrap() || x == self.max.unwrap() {
            true
        } else {
            subtree!(self, self.high(x) as usize).map_or(false, |subtree| subtree.contains(self.low(x)))
        }
    }

    // This is a helper method for find_next. It searches for an element in the subtree
    // corresponding to the given value.
    fn find_in_subtree(&self, x: u64) -> Option<u64> {
        // Subtree not present - we need to look in a different cluster. Since universe > 2, we
        // know the summary exists.
        summary!(self).find_next(self.high(x)).map_or(None, |next_index| {
            Some(self.index(next_index, subtree!(self, next_index as usize).unwrap().min.unwrap()))
        })
    }

    /// Finds the next highest value in the tree, or None if it doesn't exist.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use self::veb_rs::VEBTree;
    /// let mut tree = VEBTree::new(10).unwrap();
    /// tree.insert(5);
    /// assert_eq!(5, tree.find_next(2).unwrap());
    /// assert!(!tree.find_next(5).is_some());
    /// ```
    ///
    /// # Runtime
    /// `O(log(log(U)))`
    pub fn find_next(&self, x: u64) -> Option<u64> {
        // There are 2 base cases: 1, if the tree is empty, then there is no next value;
        if self.is_empty() {
            None
        } else if self.universe == 2 {
            // 2, if the Universe Size is 2, x is 0, and the max is 1, then there is a next value.
            // Otherwise, there is not.
            if x == 0 && self.max == Some(1) {
                Some(1)
            } else {
                None
            }
        // We know that the minimum is present, since the tree is not empty.
        } else if x < self.min.unwrap() {
            self.min
        } else {
            let subtree_index = self.high(x);
            let low = self.low(x);
            // We check the appropriate subtree for the next value.
            subtree!(self, subtree_index as usize).map_or_else(|| self.find_in_subtree(x), |subtree| {
                // We can check right away: if the subtree's maximum is higher than the portion of
                // the value that would be stored in the subtree, then there must be a next value
                // in that subtree.
                if low < subtree!(self, subtree_index as usize).unwrap().max.unwrap() {
                    Some(self.index(subtree_index, subtree.find_next(low).unwrap()))
                } else {
                    self.find_in_subtree(x)
                }
            })
        }
    }

    // Helper function for insert; inserting into an empty tree is different than inserting into a
    // tree which already has as least one element.
    fn empty_insert(&mut self, x: u64) {
        self.min = Some(x);
        self.max = Some(x);
    }

    /// Inserts a value into the tree.
    ///
    /// # Panics
    ///
    /// Panics if the value to insert is greater than or equal to the tree's universe size.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use self::veb_rs::VEBTree;
    /// let mut tree = VEBTree::new(10).unwrap();
    /// assert!(!tree.contains(5));
    /// tree.insert(5);
    /// assert!(tree.contains(5));
    /// ```
    ///
    /// # Runtime
    /// `O(log(log(U)))`
    pub fn insert(&mut self, mut x: u64) {
        assert!(x < self.universe);

        if self.is_empty() {
            self.empty_insert(x);
        } else {
            // Min and max must be present since the tree is not empty.
            if self.min == self.max {
                if x < self.min.unwrap() {
                    self.min = Some(x);
                }
                if x > self.max.unwrap() {
                    self.max = Some(x);
                }
            }
            // If we have a new minimum value, then we should set our minimum to that value and
            // insert our old minimum into the subtree.
            if x < self.min.unwrap() {
                let swap = self.min.unwrap();
                self.min = Some(x);
                x = swap;
            }
            // It's simpler for maxima; just replace it.
            if x > self.max.unwrap() {
                self.max = Some(x);
            }
            // Insert the value into a subtree.
            let subtree_index = self.high(x);
            let low = self.low(x);
            let subtree = &mut self.children[subtree_index as usize];
            match *subtree {
                Some(ref mut subtree) => subtree.insert(low),
                None => {
                    debug_assert!(self.summary.is_some());
                    let mut new_tree = VEBTree::new(self.sqrt_universe).unwrap();
                    new_tree.empty_insert(low);
                    std::mem::replace(subtree, Some(new_tree));
                    summary_mut!(self).insert(subtree_index);
                }
            }
        }
    }

    /// Removes an element from the tree, returning true if the element was present to remove.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use self::veb_rs::VEBTree;
    /// let mut tree = VEBTree::new(10).unwrap();
    /// tree.insert(4);
    /// assert!(tree.contains(4));
    /// assert!(tree.delete(4));
    /// assert!(!tree.contains(4));
    /// assert!(!tree.delete(4));
    /// ```
    ///
    /// # Runtime
    /// `O(log(log(U)))`
    pub fn delete(&mut self, mut x: u64) -> bool {
        // 2 base cases: 1, the tree has no elements, so we return false;
        if self.is_empty() {
            false
        } else if self.min == self.max && self.min.unwrap() == x {
            // 2, the tree has one element, so we remove it.
            self.min = None;
            self.max = None;
            true
        } else {
            // If we are removing the minimum, we need to determine the new one.
            if self.min.unwrap() == x {
                self.min = if summary!(self).is_empty() {
                    self.max
                } else {
                    // We need to delete the element we're replacing the minimum with.
                    x = subtree!(self, summary!(self).min.unwrap() as usize).unwrap().min.unwrap();
                    Some(x)
                }
            }
            // If we are removing the maximum, we need to determine the new one.
            if self.max.unwrap() == x {
                self.max = if summary!(self).is_empty() {
                    self.min
                } else {
                    subtree!(self, summary!(self).max.unwrap() as usize).unwrap().max
                }
            }
            // If we have subtrees, then we need to remove the element from them as well.
            if !summary!(self).is_empty() {
                let subtree_index = self.high(x);
                let low = self.low(x);
                let mut subtree = &mut self.children[subtree_index as usize];
                let result = subtree.as_mut().unwrap().delete(low);

                // If the subtree is empty now, we need to remove it from the summary, and stop
                // holding memory for it.
                if subtree.as_ref().unwrap().is_empty() {
                    subtree.take();
                    summary_mut!(self).delete(subtree_index);
                }
                result
            } else {
                true
            }
        }
    }

    /// Creates an iterator over the tree.
    ///
    /// # Examples
    /// ```rust
    /// # use self::veb_rs::VEBTree;
    /// let mut tree = VEBTree::new(10).unwrap();
    /// tree.insert(2);
    /// tree.insert(4);
    /// tree.insert(5);
    ///
    /// let result: Vec<u64> = tree.iter().collect();
    /// assert!(vec![2, 4, 5] == result);
    /// ```
    ///
    /// References to a VEBTree also implement IntoIterator, so it is possible to iterate like
    /// this:
    ///
    /// ```rust
    /// # use self::veb_rs::VEBTree;
    /// let mut tree = VEBTree::new(10).unwrap();
    /// tree.insert(2);
    /// tree.insert(4);
    /// tree.insert(5);
    /// for element in &tree {
    ///     println!("VEBTree element: {}", element);
    /// }
    /// // Prints:
    /// // VEBTree element: 2
    /// // VEBTree element: 4
    /// // VEBTree element: 5
    /// ```
    pub fn iter(&self) -> VEBTreeIterator {
        VEBTreeIterator {
            tree: self,
            done: false,
            prev: Some(0),
        }
    }
}

impl<'a> IntoIterator for &'a VEBTree {
    type Item = u64;
    type IntoIter = VEBTreeIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// An iterator over a `VEBTree`.
#[derive(Debug)]
pub struct VEBTreeIterator<'a> {
    tree: &'a VEBTree,
    done: bool,
    prev: Option<u64>,
}

impl<'a> Iterator for VEBTreeIterator<'a> {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        match self.prev {
            Some(prev) => {
                self.prev = self.tree.find_next(prev);
                self.prev
            },
            None => None
        }
    }
}

#[test]
fn creation() {
    // Test the first 1000 universe sizes.
    for i in 2..1000 {
        assert!(VEBTree::new(i).is_ok());
    }
}

#[test]
fn creation_fail() {
    // Inclusive ranges are experimental, so put n + 1 here.
    for i in 0..2 {
        assert!(VEBTree::new(i).is_err());
    }
}

#[test]
fn high_low_index() {
    let tree = VEBTree::new(64).unwrap(); // m = 6
    assert_eq!(5, tree.high(42));
    assert_eq!(2, tree.low(42));
    assert_eq!(42, tree.index(5, 2));

    let tree = VEBTree::new(128).unwrap(); // m = 7
    assert_eq!(6, tree.high(100));
    assert_eq!(4, tree.low(100));
    assert_eq!(100, tree.index(6, 4));
}

#[test]
fn insertion_and_contains() {
    for i in 2..1000 {
        let mut tree = VEBTree::new(i).unwrap();

        // Again, these are exclusive ranges, so 0..i + 1 is 0, 1, 2, ..., i - 1
        for j in 0..i {
            assert!(!tree.contains(j));

            tree.insert(j);

            for k in 0..j + 1 {
                assert!(tree.contains(k));
            }
        }
    }
}

#[test]
fn is_empty() {
    for i in 2..1000 {
        let mut tree = VEBTree::new(i).unwrap();
        assert!(tree.is_empty());
        tree.insert(i - 1);
        assert!(!tree.is_empty());
        tree.delete(i - 1);
        assert!(tree.is_empty());
        tree.insert(0);
        assert!(!tree.is_empty());
        tree.delete(0);
        assert!(tree.is_empty());
    }
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
    for i in 3..1000 {
        let mut tree = VEBTree::new(i).unwrap();
        let test_val = i / 2;
        let test_val_2 = test_val + 1;

        assert!(!tree.contains(test_val));
        assert!(!tree.contains(test_val_2));

        tree.insert(test_val);
        tree.insert(test_val_2);
        assert!(tree.contains(test_val));
        assert!(tree.contains(test_val_2));

        tree.delete(test_val);
        assert!(!tree.contains(test_val));
        assert!(tree.contains(test_val_2));

        tree.delete(test_val_2);
        assert!(!tree.contains(test_val));
        assert!(!tree.contains(test_val_2));
    }
}
