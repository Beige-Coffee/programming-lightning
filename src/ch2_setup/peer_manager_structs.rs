#![allow(dead_code, unused_imports, unused_variables, unused_must_use)]
use crate::ch3_keys::exercises::SimpleKeysManager as KeysManager;
use crate::internal;
use bitcoin::secp256k1::{self, PublicKey, Secp256k1};
use internal::events::MessageType;
use internal::messages::Message;
use internal::events::MessageSendEvent;
use internal::messages::{AcceptChannel, ChannelReady, FundingCreated, FundingSigned, OpenChannel,
                         OnionMessage, NodeAnnouncement, ChannelAnnouncement};
use lightning::ln::types::ChannelId;
use std::collections::HashMap;
use std::ops::Deref;
use bitcoin::secp256k1::{ecdsa::Signature};
use bitcoin::secp256k1::ffi::Signature as FFISignature;
use crate::internal::helper::{ pubkey_from_private_key,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Peer {
    pub public_key: PublicKey,
}

impl Peer {
    pub fn decrypt_message(self, data: &[u8]) -> Message {
        match data.get(0) {
            Some(0x00) => {
                let message = OpenChannel {
                    channel_value_satoshis: 100_000_000,
                };
                Message::OpenChannel(message)
            }
            Some(0x01) => {
                let message = NodeAnnouncement {
                  signature: Signature::from(unsafe { FFISignature::new() }),
                  contents: [1; 32]
                };
                Message::NodeAnnouncement(message)
            }
            Some(0x02) => {
                let message = OnionMessage {
                  blinding_point: pubkey_from_private_key(&[0x01; 32]),
                  onion_routing_packet: [1; 32],
                };
                Message::OnionMessage(message)
            }
            _ => panic!("Unknown message type"),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct SocketDescriptor {
    pub pubkey: PublicKey,
    pub addr: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChannelMessageHandler {}
impl ChannelMessageHandler {
    /// Handle an incoming `open_channel` message from the given peer.
    pub fn handle_open_channel(&self, their_node_id: PublicKey, msg: &OpenChannel) {
        unimplemented!()
    }
    /// Handle an incoming `accept_channel` message from the given peer.
    pub fn handle_accept_channel(&self, their_node_id: PublicKey, msg: &AcceptChannel) {
        unimplemented!()
    }

    /// Handle an incoming `funding_created` message from the given peer.
    pub fn handle_funding_created(&self, their_node_id: PublicKey, msg: &FundingCreated) {
        unimplemented!()
    }

    /// Handle an incoming `funding_signed` message from the given peer.
    pub fn handle_funding_signed(&self, their_node_id: PublicKey, msg: &FundingSigned) {
        unimplemented!()
    }

    /// Handle an incoming `channel_ready` message from the given peer.
    pub fn handle_channel_ready(&self, their_node_id: PublicKey, msg: &ChannelReady) {
        unimplemented!()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoutingMessageHandler {}
impl RoutingMessageHandler {
    /// Handle an incoming `node_announcement` message, returning `true` if it should be forwarded on,
    /// `false` or returning an `Err` otherwise.
    ///
    /// If `their_node_id` is `None`, the message was generated by our own local node.
    pub fn handle_node_announcement(
        &self,
        their_node_id: Option<PublicKey>,
        msg: &NodeAnnouncement,
    ) {
        unimplemented!()
    }
    /// Handle a `channel_announcement` message, returning `true` if it should be forwarded on, `false`
    /// or returning an `Err` otherwise.
    ///
    /// If `their_node_id` is `None`, the message was generated by our own local node.
    pub fn handle_channel_announcement(
        &self,
        their_node_id: Option<PublicKey>,
        msg: &ChannelAnnouncement,
    ) {
        unimplemented!()
    }
}

/// A handler for received [`OnionMessage`]s and for providing generated ones to send.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OnionMessageHandler {}
impl OnionMessageHandler {
    /// Handle an incoming `onion_message` message from the given peer.
    pub fn handle_onion_message(&self, peer_node_id: PublicKey, msg: &OnionMessage) {
        unimplemented!()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MessageHandler {
    pub channel_message_handler: ChannelMessageHandler,
    pub route_message_handler: RoutingMessageHandler,
    pub onion_message_handler: OnionMessageHandler,
}

// Updated PeerManager with generics
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PeerManager {
    pub peers: HashMap<SocketDescriptor, Peer>,
    pub pending_msg_events: Vec<MessageSendEvent>,
    pub message_handler: MessageHandler,
    pub node_signer: KeysManager,
    pub secp_ctx: Secp256k1<secp256k1::SignOnly>,
}

// Test constructor
impl PeerManager {
    pub fn new() -> Self {
        let secp_ctx = Secp256k1::signing_only();

        let seed = [1_u8; 32];
        let child_index: usize = 0;
        let keys_manager = KeysManager::new(seed);

        let message_handler = MessageHandler {
            channel_message_handler: ChannelMessageHandler {},
            route_message_handler: RoutingMessageHandler {},
            onion_message_handler: OnionMessageHandler {},
        };

        PeerManager {
            peers: HashMap::new(),
            pending_msg_events: Vec::new(),
            message_handler,
            node_signer: keys_manager,
            secp_ctx,
        }
    }
}
