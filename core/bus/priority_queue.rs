//! Priority lane definitions for the in-process bus.

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum PriorityLane {
    P0,
    P1,
    P2,
    P3,
    P4,
    P5,
    P6,
}

impl PriorityLane {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::P0 => "P0",
            Self::P1 => "P1",
            Self::P2 => "P2",
            Self::P3 => "P3",
            Self::P4 => "P4",
            Self::P5 => "P5",
            Self::P6 => "P6",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::P0 => "safety interrupt",
            Self::P1 => "active action correction",
            Self::P2 => "active goal relevance",
            Self::P3 => "contradiction/anomaly",
            Self::P4 => "memory maintenance",
            Self::P5 => "curiosity/background learning",
            Self::P6 => "archival/compression",
        }
    }
}
