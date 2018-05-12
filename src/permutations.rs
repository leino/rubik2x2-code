pub const IDENTITY: [u8; 8] = [0, 1, 2, 3, 4, 5, 6, 7];

pub fn inverse(p: [u8; 8]) -> [u8; 8] {
    let mut q: [u8; 8] = [0; 8];
    for i in 0..8 {
        let mut j = 0;
        while j < 8 {
            if p[j] == i {
                break;
            } else {
                j += 1;
            }
        }
        q[i as usize] = j as u8;
    }
    q
}

pub fn apply(a: &mut [u8; 8], p: &[u8; 8]) {
    let orig = a.clone();
    for i in 0..8 {
        a[p[i] as usize] = orig[i];
    }
}

pub struct CycleIterator {
    permutation: [u8; 8],
    index_used: [bool; 8],
    optional_next_index: Option<usize>,
}

pub struct TranspositionIterator {
    cycle_iterator: CycleIterator,
    swap_index: usize,
    first: bool,
}

impl TranspositionIterator {
    pub fn new(p: [u8; 8]) -> TranspositionIterator {
        TranspositionIterator {
            cycle_iterator: CycleIterator::new(p),
            first: true,
            swap_index: 0,
        }
    }
}

impl Iterator for TranspositionIterator {
    type Item = (usize, usize);
    fn next(&mut self) -> Option<Self::Item> {
        while let Some((last, index)) = self.cycle_iterator.next() {
            let first = self.first;
            self.first = last;
            if first {
                self.swap_index = index;
            } else {
                return Some((self.swap_index, index));
            }
        }
        None
    }
}

impl CycleIterator {
    fn new(p: [u8; 8]) -> CycleIterator {
        CycleIterator {
            permutation: p,
            index_used: [false; 8],
            optional_next_index: Some(0),
        }
    }
}

impl Iterator for CycleIterator {
    type Item = (bool, usize);
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(index) = self.optional_next_index {
            self.index_used[index] = true;
            let next_index = self.permutation[index] as usize;
            let mut last = false;
            self.optional_next_index =
                if self.index_used[next_index] {
                    last = true;
                    self.index_used.iter().position(|x| !x)
                } else {
                    Some(next_index)
                };
            Some((last, index))
        } else {
            None
        }
    }
}
