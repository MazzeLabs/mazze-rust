

pub mod message;
pub mod network_event;
pub mod network_sender;
pub mod request_manager;
pub mod sync_protocol;

use network::{service::ProtocolVersion, ProtocolId};

pub const HSB_PROTOCOL_ID: ProtocolId = *b"mzhsb"; // HotStuff Synchronization Protocol
pub const HSB_PROTOCOL_VERSION: ProtocolVersion = ProtocolVersion(1);
