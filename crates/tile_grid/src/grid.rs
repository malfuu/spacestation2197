use std::array;

use bevy::prelude::*;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A statically sized 2D grid, mostly for chunks
/// sigh... AFAIK you must manually insert and ensure A = R * C
/// until <https://github.com/rust-lang/rust/issues/76560> drops.
#[derive(Debug, Clone)]
pub struct UnsizedBaseGrid<T, const R: usize, const C: usize, const A: usize> {
    // The flat array living on the stack
    data: [T; A], // perhaps using [[T; C]; R] might be better.
}

impl<T, const R: usize, const C: usize, const A: usize> UnsizedBaseGrid<T, R, C, A> {
    pub const fn rows() -> usize {
        R
    }

    pub const fn columns() -> usize {
        C
    }

    pub const fn area() -> usize {
        A
    }

    pub fn from_value(value: T) -> Self
    where
        T: Copy,
    {
        // Sanity check
        assert_eq!(R * C, A, "Invalid chunk sizes."); // TODO perhaps make this compile time.

        Self { data: [value; A] }
    }

    #[inline]
    #[must_use]
    fn position_to_index(pos: UVec2) -> Option<usize> {
        if pos.x >= C as u32 || pos.y >= R as u32 {
            return None;
        }

        Some((pos.y as usize * C) + pos.x as usize)
    }

    #[inline]
    #[must_use]
    fn index_to_position(index: usize) -> UVec2 {
        let x = (index % C) as u32;
        let y = (index / C) as u32;
        UVec2::new(x, y)
    }

    #[must_use]
    pub fn get(&self, pos: UVec2) -> Option<&T> {
        let index = Self::position_to_index(pos)?;

        self.data.get(index)
    }

    #[must_use]
    pub fn get_mut(&mut self, pos: UVec2) -> Option<&mut T> {
        let index = Self::position_to_index(pos)?;
        self.data.get_mut(index)
    }

    #[must_use]
    pub fn get_two(&self, pos_a: UVec2, pos_b: UVec2) -> Option<(&T, &T)> {
        let a = self.get(pos_a)?;
        let b = self.get(pos_b)?;
        if pos_a == pos_b {
            return None;
        }
        Some((a, b))
    }

    #[must_use]
    pub fn get_two_mut(&mut self, pos_a: UVec2, pos_b: UVec2) -> Option<(&mut T, &mut T)> {
        let idx_a = Self::position_to_index(pos_a)?;
        let idx_b = Self::position_to_index(pos_b)?;

        if idx_a == idx_b {
            return None;
        }

        // # Safety
        // idx_a != idx_b
        unsafe {
            let ptr = self.data.as_mut_ptr();
            Some((&mut *ptr.add(idx_a), &mut *ptr.add(idx_b)))
        }
    }

    pub fn set(&mut self, pos: UVec2, value: T) -> Option<T> {
        let index = Self::position_to_index(pos)?;

        if let Some(cell) = self.data.get_mut(index) {
            Some(std::mem::replace(cell, value))
        } else {
            None
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.data.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.data.iter_mut()
    }

    pub fn iter_with_pos(&self) -> impl Iterator<Item = (UVec2, &T)> {
        self.iter()
            .enumerate()
            .map(|(i, item)| (Self::index_to_position(i), item))
    }

    pub fn iter_mut_with_pos(&mut self) -> impl Iterator<Item = (UVec2, &mut T)> {
        self.iter_mut()
            .enumerate()
            .map(|(i, item)| (Self::index_to_position(i), item))
    }

    pub fn iter_row(&self, row: usize) -> Option<impl Iterator<Item = &T>> {
        self.data
            .chunks_exact(C)
            .nth(row)
            .map(|row_slice| row_slice.iter())
    }

    pub fn iter_column(&self, col: usize) -> Option<impl Iterator<Item = &T>> {
        if col >= C {
            return None;
        }

        Some(self.data.iter().skip(col).step_by(C))
    }

    pub fn iter_row_mut(&mut self, row: usize) -> Option<impl Iterator<Item = &mut T>> {
        self.data
            .chunks_exact_mut(C)
            .nth(row)
            .map(|row_slice| row_slice.iter_mut())
    }

    pub fn iter_column_mut(&mut self, col: usize) -> Option<impl Iterator<Item = &mut T>> {
        if col >= C {
            return None;
        }

        Some(self.data.iter_mut().skip(col).step_by(C))
    }
}

impl<T: Copy, const R: usize, const C: usize, const A: usize> Copy for UnsizedBaseGrid<T, R, C, A> {}

impl<T, const R: usize, const C: usize, const A: usize> Default for UnsizedBaseGrid<T, R, C, A>
where
    T: Default,
{
    fn default() -> Self {
        // sanity check
        assert_eq!(R * C, A, "Invalid chunk sizes.");
        Self {
            data: array::from_fn(|_| T::default()),
        }
    }
}

impl<V, const R: usize, const C: usize, const A: usize> UnsizedBaseGrid<Option<V>, R, C, A> {
    pub fn is_all_none(&self) -> bool {
        self.data.iter().all(|cell| cell.is_none())
    }

    pub fn set_all_none(&mut self) {
        for cell in self.data.iter_mut() {
            *cell = None;
        }
    }
}

impl<T, const R: usize, const C: usize, const A: usize> Serialize for UnsizedBaseGrid<T, R, C, A>
where
    T: Serialize,
{
    fn serialize<Ser>(&self, serializer: Ser) -> Result<Ser::Ok, Ser::Error>
    where
        Ser: Serializer,
    {
        self.data.serialize(serializer)
    }
}

impl<'de, T, const R: usize, const C: usize, const A: usize> Deserialize<'de>
    for UnsizedBaseGrid<T, R, C, A>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let vec = heapless::Vec::<T, A>::deserialize(deserializer)?;

        let data: [T; A] = vec.into_array().map_err(|_| {
            serde::de::Error::custom(format!(
                "Expected array of size {}, found different length",
                A
            ))
        })?;

        if R * C != A {
            return Err(serde::de::Error::custom("Grid dimension mismatch"));
        }

        Ok(Self { data })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "Invalid chunk sizes.")]
    fn size_mismatch_panic() {
        // R=2, C=2, A=5 (should be 4)
        let _ = UnsizedBaseGrid::<i32, 2, 2, 5>::from_value(0);
    }

    #[test]
    fn from_value_initialization() {
        let value = 42;
        let grid: UnsizedBaseGrid<i32, 2, 2, 4> = UnsizedBaseGrid::from_value(value);
        for (_, &cell) in grid.iter_with_pos() {
            assert_eq!(cell, value);
        }
    }

    #[test]
    fn default_initializion() {
        let grid: UnsizedBaseGrid<i32, 2, 2, 4> = UnsizedBaseGrid::default();
        for (_, &cell) in grid.iter_with_pos() {
            assert_eq!(cell, i32::default());
        }
    }

    #[test]
    fn pos_to_index_valid() {
        type TestGrid = UnsizedBaseGrid<i32, 3, 3, 9>;
        assert_eq!(TestGrid::position_to_index(UVec2::new(0, 0)), Some(0));
        assert_eq!(TestGrid::position_to_index(UVec2::new(2, 0)), Some(2));
        assert_eq!(TestGrid::position_to_index(UVec2::new(0, 1)), Some(3));
        assert_eq!(TestGrid::position_to_index(UVec2::new(2, 2)), Some(8));
    }

    #[test]
    fn pos_to_index_out_of_bounds() {
        // 2 rows, 3 columns, Area 6
        type TestGrid = UnsizedBaseGrid<i32, 2, 3, 6>;
        // x is column, so it must be < 3
        // y is row, so it must be < 2
        assert!(TestGrid::position_to_index(UVec2::new(3, 0)).is_none());
        assert!(TestGrid::position_to_index(UVec2::new(0, 2)).is_none());
        assert!(TestGrid::position_to_index(UVec2::new(3, 2)).is_none());
    }

    #[test]
    fn index_to_pos_inverse() {
        type TestGrid = UnsizedBaseGrid<i32, 4, 4, 16>;
        for i in 0..16 {
            let pos = TestGrid::index_to_position(i);
            let index = TestGrid::position_to_index(pos).unwrap();
            assert_eq!(i, index);
        }
    }

    #[test]
    fn get_and_set() {
        let mut grid: UnsizedBaseGrid<i32, 2, 2, 4> = UnsizedBaseGrid::from_value(0);
        let pos = UVec2::new(1, 1);

        grid.set(pos, 500);
        assert_eq!(grid.get(pos), Some(&500));
    }

    #[test]
    fn test_get_two_mut() {
        let mut grid: UnsizedBaseGrid<i32, 2, 2, 4> = UnsizedBaseGrid::from_value(0);
        let pos_a = UVec2::new(0, 0);
        let pos_b = UVec2::new(1, 1);

        if let Some((a, b)) = grid.get_two_mut(pos_a, pos_b) {
            *a = 10;
            *b = 20;
        }

        assert_eq!(*grid.get(pos_a).unwrap(), 10);
        assert_eq!(*grid.get(pos_b).unwrap(), 20);
        assert!(grid.get_two_mut(pos_a, pos_a).is_none());
    }

    #[test]
    fn set_returns_old_value() {
        let mut grid: UnsizedBaseGrid<i32, 2, 2, 4> = UnsizedBaseGrid::from_value(10);
        let pos = UVec2::new(0, 0);

        let old = grid.set(pos, 20);
        assert_eq!(old, Some(10));
        assert_eq!(grid.get(pos), Some(&20));
    }

    #[test]
    fn out_of_bounds_access_returns_none() {
        let mut grid: UnsizedBaseGrid<i32, 2, 2, 4> = UnsizedBaseGrid::default();
        let bad_pos = UVec2::new(5, 5);

        assert!(grid.get(bad_pos).is_none());
        assert!(grid.get_mut(bad_pos).is_none());
        assert!(grid.set(bad_pos, 10).is_none());
    }

    #[test]
    fn option_helpers_work() {
        let mut grid: UnsizedBaseGrid<Option<i32>, 2, 2, 4> = UnsizedBaseGrid::default();
        assert!(grid.is_all_none());

        grid.set(UVec2::new(0, 0), Some(1));
        assert!(!grid.is_all_none());

        grid.set_all_none();
        assert!(grid.is_all_none());
    }
}
