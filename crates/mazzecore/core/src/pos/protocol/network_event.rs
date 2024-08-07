

use serde::{Deserialize, Serialize};

/// Container for exchanging transactions with other Mempools.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum NetworkEvent {
    PeerConnected,
    PeerDisconnected,
}
