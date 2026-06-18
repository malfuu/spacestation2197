use std::ops::{BitAnd, BitOr, BitXor, Not};

use serde::{Deserialize, Serialize};

use crate::{CHUNK_AREA, CHUNK_SIZE, LocalTilePosition};

const _: () = assert!(CHUNK_SIZE == 16, "ChunkMask is built for CHUNK_SIZE of 16");

/// a boolean mask for a CHUNK_SIZE'd chunk
#[derive(Clone, Copy, Default, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChunkMask(pub [u64; 4]);

impl ChunkMask {
    #[inline]
    fn to_index_and_bit(index: u32) -> (usize, u64) {
        ((index / 64) as usize, 1_u64 << (index % 64))
    }

    #[inline]
    pub fn bit_to_position(index: u32) -> LocalTilePosition {
        LocalTilePosition::new(index % (CHUNK_SIZE as u32), index / (CHUNK_SIZE as u32))
    }

    #[inline]
    pub fn set_position(&mut self, pos: LocalTilePosition) {
        let index = pos.y * (CHUNK_SIZE as u32) + pos.x;
        let (array_index, bit) = Self::to_index_and_bit(index);
        self.0[array_index] |= bit;
    }

    #[inline]
    pub fn clear_position(&mut self, pos: LocalTilePosition) {
        let index = pos.y * (CHUNK_SIZE as u32) + pos.x;
        let (array_index, bit) = Self::to_index_and_bit(index);
        self.0[array_index] &= !bit;
    }

    #[inline]
    pub fn has_position(&self, pos: LocalTilePosition) -> bool {
        let index = pos.y * (CHUNK_SIZE as u32) + pos.x;
        let (array_index, bit) = Self::to_index_and_bit(index);
        (self.0[array_index] & bit) != 0
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0 == [0; 4]
    }

    #[inline]
    pub fn has_index(&self, index: u32) -> bool {
        let (array_index, bit) = Self::to_index_and_bit(index);
        (self.0[array_index] & bit) != 0
    }

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
        Self([
            self.0[0] | rhs.0[0],
            self.0[1] | rhs.0[1],
            self.0[2] | rhs.0[2],
            self.0[3] | rhs.0[3],
        ])
    }
}

impl BitAnd for ChunkMask {
    type Output = Self;
    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        Self([
            self.0[0] & rhs.0[0],
            self.0[1] & rhs.0[1],
            self.0[2] & rhs.0[2],
            self.0[3] & rhs.0[3],
        ])
    }
}

impl BitXor for ChunkMask {
    type Output = Self;
    #[inline]
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self([
            self.0[0] ^ rhs.0[0],
            self.0[1] ^ rhs.0[1],
            self.0[2] ^ rhs.0[2],
            self.0[3] ^ rhs.0[3],
        ])
    }
}

impl Not for ChunkMask {
    type Output = Self;
    #[inline]
    fn not(self) -> Self::Output {
        Self([!self.0[0], !self.0[1], !self.0[2], !self.0[3]])
    }
}
