use serde::{Deserialize, Serialize};

use super::grid::Grid;

/// What the board server last knew the board showed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Snapshot {
    /// Board contents or none before the server first reads or sends them.
    pub grid: Option<Grid>,

    /// Whether the server's last contact with the board worked.
    pub live: bool,
}
