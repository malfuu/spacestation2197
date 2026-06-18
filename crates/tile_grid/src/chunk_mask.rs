use std::ops::{BitAnd, BitOr, BitXor, Not};

use serde::{Deserialize, Serialize};

use crate::{CHUNK_AREA, CHUNK_SIZE, LocalTilePosition};

/// a boolean mask for a CHUNK_SIZE'd chunk
#[derive(Clone, Copy, Default, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChunkMask(pub u64);

impl ChunkMask {
    #[inline]
    pub fn position_to_bit(pos: LocalTilePosition) -> u64 {
        1_u64 << (pos.y * (CHUNK_SIZE as u32) + pos.x)
    }

    #[inline]
    pub fn bit_to_position(index: u32) -> LocalTilePosition {
        LocalTilePosition::new(index % (CHUNK_SIZE as u32), index / (CHUNK_SIZE as u32))
    }

    #[inline]
    pub fn set_position(&mut self, pos: LocalTilePosition) {
        self.0 |= Self::position_to_bit(pos);
    }

    #[inline]
    pub fn clear_position(&mut self, pos: LocalTilePosition) {
        self.0 &= !Self::position_to_bit(pos);
    }

    #[inline]
    pub fn has_position(&self, pos: LocalTilePosition) -> bool {
        (self.0 & Self::position_to_bit(pos)) != 0
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }

    #[inline]
    pub fn has_index(&self, index: u32) -> bool {
        (self.0 & (1_u64 << index)) != 0
    }

    pub fn iter_positions(&self) -> impl Iterator<Item = LocalTilePosition> {
        let limit = CHUNK_AREA as u32;
        (0..limit)
            .filter(|&i| self.has_index(i))
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
        Self(!self.0)
    }
}
