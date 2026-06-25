//! Atmospheric combustion/fire zones.
use serde::{Deserialize, Serialize};

/// A localized fire burning within a gas mixture.
/// Usually utilized in combustion reactions.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Hotspot {
    /// Fraction of the total mixture volume occupied by the hotspot.
    pub ratio: f32,
}
