//! Connection Gate for the libp2p Gossip Swarm.

use std::net::IpAddr;

use ipnet::IpNet;
use libp2p::{Multiaddr, PeerId};

use crate::{Connectedness, DialError};

/// Connection Gate trait for isolating peer connection logic.
pub trait ConnectionGate {
    /// Checks if a peer is allowed to connect to the gossip swarm.
    fn can_dial(&mut self, peer_id: &Multiaddr) -> Result<(), DialError>;

    /// Returns the [`Connectedness`] for a given peer id.
    fn connectedness(&self, peer_id: &PeerId) -> Connectedness;

    /// Marks an address as currently being dialed.
    fn dialing(&mut self, addr: &Multiaddr);

    /// Marks an address as dialed.
    fn dialed(&mut self, addr: &Multiaddr);

    /// Removes a peer id from the current dials set.
    fn remove_dial(&mut self, peer: &PeerId);

    /// Checks if a peer can be removed from the gossip swarm.
    fn can_disconnect(&self, peer_id: &Multiaddr) -> bool;

    /// Blocks a given peer id.
    fn block_peer(&mut self, peer_id: &PeerId);

    /// Unblocks a given peer id.
    fn unblock_peer(&mut self, peer_id: &PeerId);

    /// Lists the blocked peers.
    fn list_blocked_peers(&self) -> Vec<PeerId>;

    /// Blocks a given ip address from connecting to the gossip swarm.
    fn block_addr(&mut self, ip: IpAddr);

    /// Unblocks a given ip address.
    fn unblock_addr(&mut self, ip: IpAddr);

    /// Lists all blocked ip addresses.
    fn list_blocked_addrs(&self) -> Vec<IpAddr>;

    /// Blocks a subnet from connecting to the gossip swarm.
    fn block_subnet(&mut self, subnet: IpNet);

    /// Unblocks a subnet.
    fn unblock_subnet(&mut self, subnet: IpNet);

    /// Lists all blocked subnets.
    fn list_blocked_subnets(&self) -> Vec<IpNet>;

    /// Protects a peer from being disconnected.
    fn protect_peer(&mut self, peer_id: PeerId);

    /// Unprotects a peer.
    fn unprotect_peer(&mut self, peer_id: PeerId);

    /// Lists all protected peers.
    fn list_protected_peers(&self) -> Vec<PeerId>;
}
