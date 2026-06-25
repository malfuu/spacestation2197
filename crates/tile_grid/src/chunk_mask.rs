//! Bitmask implementation for use with chunks.
use std::ops::{BitAnd, BitOr, BitXor, Not};

use serde::{Deserialize, Serialize};
use wide::u64x4;

use crate::{CHUNK_AREA, CHUNK_SIZE, LocalTilePosition};

const _: () = assert!(CHUNK_SIZE == 16, "ChunkMask is built for CHUNK_SIZE of 16");

/// A boolean mask for a grid chunk of `CHUNK_SIZE` dimension.
/// Under the hood, this uses four 64-bit integers.
#[derive(Clone, Copy, Default, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChunkMask(pub u64x4);

// investigate into use u16x16 instead of u64x4.

impl ChunkMask {
    #[inline]
    fn to_index_and_bit(index: u32) -> (usize, u64) {
        let array_index = (index / 64) as usize;
        let bit = 1_u64 << (array_index % 64);
        (array_index, bit)
    }

    /// Bit index to 2D tile position
    #[inline]
    pub fn bit_to_position(index: u32) -> LocalTilePosition {
        LocalTilePosition::new(index % (CHUNK_SIZE as u32), index / (CHUNK_SIZE as u32))
    }

    /// Sets the bit to `true`.
    #[inline]
    pub fn set_position(&mut self, pos: LocalTilePosition) {
        let index = pos.y * (CHUNK_SIZE as u32) + pos.x;
        let (array_index, bit) = Self::to_index_and_bit(index);

        let mut arr = self.0.to_array();
        arr[array_index] |= bit;
        self.0 = u64x4::from(arr);
    }

    /// Sets the bit to `false`.
    #[inline]
    pub fn clear_position(&mut self, pos: LocalTilePosition) {
        let index = pos.y * (CHUNK_SIZE as u32) + pos.x;
        let (array_index, bit) = Self::to_index_and_bit(index);

        let mut arr = self.0.to_array();
        arr[array_index] &= !bit;
        self.0 = u64x4::from(arr);
    }

    /// Returns bit value at a given position.
    #[inline]
    pub fn has_position(&self, pos: LocalTilePosition) -> bool {
        let index = pos.y * (CHUNK_SIZE as u32) + pos.x;
        let (array_index, bit) = Self::to_index_and_bit(index);
        (self.0.to_array()[array_index] & bit) != 0
    }

    /// Checks if all bits in the mask are `false`.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.none()
    }

    /// Returns bit value at a given index.
    #[inline]
    pub fn has_index(&self, index: u32) -> bool {
        let (array_index, bit) = Self::to_index_and_bit(index);
        (self.0.to_array()[array_index] & bit) != 0
    }

    /// Iterates over every bit set to `true`.
    pub fn iter_positions(&self) -> impl Iterator<Item = LocalTilePosition> + '_ {
        let limit = CHUNK_AREA as u32;
        (0..limit)
            .filter(move |&i| self.has_index(i))
            .map(Self::bit_to_position)
    }
}

impl BitOr for ChunkMask {
    type Output = Self;
    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitAnd for ChunkMask {
    type Output = Self;
    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitXor for ChunkMask {
    type Output = Self;
    #[inline]
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl Not for ChunkMask {
    type Output = Self;
    #[inline]
    fn not(self) -> Self::Output {
        Self(self.0 ^ u64x4::from([u64::MAX; 4]))
    }
}
