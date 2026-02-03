//! Event Handling Module.

use libp2p::{gossipsub, identify, ping};

/// High-level events emitted by the gossip networking system.
#[derive(Debug)]
pub enum Event {
    /// Network connectivity check event from the ping protocol.
    #[allow(dead_code)]
    Ping(ping::Event),

    /// `GossipSub` mesh networking event.
    Gossipsub(Box<gossipsub::Event>),

    /// Peer identification protocol event.
    Identify(Box<identify::Event>),

    /// Stream protocol event for request-response communication.
    Stream,
}

impl From<ping::Event> for Event {
    fn from(value: ping::Event) -> Self {
        Self::Ping(value)
    }
}

impl From<gossipsub::Event> for Event {
    fn from(value: gossipsub::Event) -> Self {
        Self::Gossipsub(Box::new(value))
    }
}

impl From<identify::Event> for Event {
    fn from(value: identify::Event) -> Self {
        Self::Identify(Box::new(value))
    }
}

impl From<()> for Event {
    fn from(_value: ()) -> Self {
        Self::Stream
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_conversion() {
        let gossipsub_event = libp2p::gossipsub::Event::Message {
            propagation_source: libp2p::PeerId::random(),
            message_id: libp2p::gossipsub::MessageId(vec![]),
            message: libp2p::gossipsub::Message {
                source: None,
                data: vec![],
                sequence_number: None,
                topic: libp2p::gossipsub::TopicHash::from_raw("test"),
            },
        };
        let event = Event::from(gossipsub_event);
        match event {
            Event::Gossipsub(e) => {
                if !matches!(*e, libp2p::gossipsub::Event::Message { .. }) {
                    panic!("Event conversion failed");
                }
            }
            _ => panic!("Event conversion failed"),
        }
    }

    #[test]
    fn test_event_conversion_ping() {
        let ping_event = ping::Event {
            peer: libp2p::PeerId::random(),
            connection: libp2p::swarm::ConnectionId::new_unchecked(0),
            result: Ok(core::time::Duration::from_secs(1)),
        };
        let event = Event::from(ping_event);
        match event {
            Event::Ping(_) => {}
            _ => panic!("Event conversion failed"),
        }
    }
}
