use rand::RngCore;
use rand::rngs::OsRng;
use crate::{RandomSequence, RandomSequenceBuilder};

// TODO: continue the implementation for PermutedSlice.

pub trait PermuteSlice<'a, T> {
    /// Randomly permute a Slice, returning a [PermutedSlice].
    ///
    /// Only available with the `rand` feature.
    #[cfg(feature = "rand")]
    fn permute(&'a self, rng: &'a mut OsRng) -> PermutedSlice<'a, T>;

    /// Randomly permute a SliceMut, returning a [PermutedSliceMut].
    ///
    /// Only available with the `rand` feature.
    #[cfg(feature = "rand")]
    fn permute_mut(&'a self, rng: &'a mut OsRng) -> PermutedSliceMut<'a, T>;

    /// Randomly permute a Slice with a specific seed, returning a [PermutedSlice].
    fn permute_with_seed(&'a self, seed: u64) -> PermutedSlice<'a, T>;

    /// Randomly permute a SliceMut with a specific seed, returning a [PermutedSliceMut].
    fn permute_mut_with_seed(&'a self, seed: u64) -> PermutedSliceMut<'a, T>;
}

#[derive(Debug, Clone)]
pub struct PermutedSlice<'a, T> {
    slice: &'a [T],
    sequence: RandomSequence<usize>,
}

#[derive(Debug, Clone)]
pub struct PermutedSliceMut<'a, T> {
    slice: &'a [T],
    sequence: RandomSequence<usize>,
}

#[derive(Debug, Clone)]
pub struct PermutedSliceIterator<'a, T> {
    slice: &'a [T],
    sequence: RandomSequence<usize>,
}

fn permute_inner<T>(slice: &[T], builder: RandomSequenceBuilder<usize>) -> PermutedSlice<T> {
    let sequence = builder
        .with_max(slice.len() - 1)
        .into_iter();

    PermutedSlice {
        slice,
        sequence,
    }
}

fn permute_mut_inner<T>(slice: &[T], builder: RandomSequenceBuilder<usize>) -> PermutedSliceMut<T> {
    let sequence = builder
        .with_max(slice.len() - 1)
        .into_iter();

    PermutedSliceMut {
        slice,
        sequence,
    }
}

impl<'a, T> PermuteSlice<'a, T> for &'a [T] {
    #[cfg(feature = "rand")]
    fn permute(&'a self, rng: &'a mut OsRng) -> PermutedSlice<'a, T> {
        permute_inner(self, RandomSequenceBuilder::<usize>::rand(rng))
    }

    #[cfg(feature = "rand")]
    fn permute_mut(&'a self, rng: &'a mut OsRng) -> PermutedSliceMut<'a, T> {
        permute_mut_inner(self, RandomSequenceBuilder::<usize>::rand(rng))
    }

    fn permute_with_seed(&'a self, seed: u64) -> PermutedSlice<'a, T> {
        permute_inner(self, RandomSequenceBuilder::<usize>::seed(seed))
    }

    fn permute_mut_with_seed(&'a self, seed: u64) -> PermutedSliceMut<'a, T> {
        permute_mut_inner(self, RandomSequenceBuilder::<usize>::seed(seed))
    }
}

impl<'a, T> PermutedSlice<'a, T> {
    fn get(&self, index: usize) -> Option<&T> {
        self.slice.get(self.sequence.n(index))
    }
}

impl<'a, T> IntoIterator for PermutedSlice<'a, T> {
    type Item = &'a T;
    type IntoIter = PermutedSliceIterator<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        PermutedSliceIterator {
            slice: self.slice,
            sequence: self.sequence.config.into_iter(),
        }
    }
}

impl<'a, T> Iterator for PermutedSliceIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.sequence.index() >= self.slice.len() {
            return None;
        }
        let index = self.sequence.next();
        Some(&self.slice[index])
    }
}

#[cfg(test)]
mod tests {
    use std::vec::Vec;

    use super::PermuteSlice;

    #[test]
    fn test_create() {
        let slice: &[i32] = &[1, 2, 3, 4, 5];
        let permuted = slice.permute_with_seed(0);
        let values: Vec<_> = permuted.clone().into_iter().take(5).collect();
        assert_eq!(values, &[&1, &3, &5, &4, &2]);
        assert_eq!(permuted.get(1), Some(&3));
        assert_eq!(permuted.get(3), Some(&4));
    }
}
