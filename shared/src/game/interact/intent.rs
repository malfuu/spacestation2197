//! All mobs can interact with an intent, although this intent might not correspond to a special
//! action. The client is responsible to send intent the interaction had.
use serde::{Deserialize, Serialize};

/// What intent an interaction was made with.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub enum Intent {
    #[default]
    Passive,
    Aggressive,
}

impl Intent {
    pub fn is_passive(&self) -> bool {
        *self == Intent::Passive
    }

    pub fn is_aggressive(&self) -> bool {
        *self == Intent::Aggressive
    }

    pub fn switch(&mut self) -> Self {
        *self = match self {
            Intent::Passive => Intent::Aggressive,
            Intent::Aggressive => Intent::Passive,
        };

        *self
    }
}
