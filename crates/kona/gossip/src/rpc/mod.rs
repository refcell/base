//! RPC API types and request handling for P2P administration.

mod request;
pub use request::P2pRpcRequest;

mod types;
pub use types::{
    Connectedness, Direction, GossipScores, PeerCount, PeerDump, PeerInfo, PeerScores, PeerStats,
    ReqRespScores, TopicScores,
};
