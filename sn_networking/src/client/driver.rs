use crate::circular_vec::CircularVec;
use crate::client::commands::{LocalSwarmCmd, NetworkSwarmCmd};
use crate::driver::{NodeBehaviour, PendingGetClosestType, PendingGetRecord};
use crate::network_discovery::NetworkDiscovery;
use crate::{multiaddr_pop_p2p, NetworkEvent};
use futures::StreamExt;
use libp2p::kad::QueryId;
#[cfg(feature = "local")]
use libp2p::mdns;
use libp2p::request_response::OutboundRequestId;
use libp2p::swarm::dial_opts::{DialOpts, PeerCondition};
use libp2p::swarm::{ConnectionId, DialError};
use libp2p::{Multiaddr, PeerId, Swarm};
use sn_protocol::messages::Response;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::time::Instant;
use tokio::spawn;
use tokio::sync::{mpsc, oneshot};

type PendingGetClosest = HashMap<QueryId, (PendingGetClosestType, Vec<PeerId>)>;

pub struct SwarmDriverClient {
    pub swarm: Swarm<NodeBehaviour>,
    /// When true, we don't filter our local addresses
    pub local: bool,
    pub network_cmd_sender: mpsc::Sender<NetworkSwarmCmd>,
    pub local_cmd_sender: mpsc::Sender<LocalSwarmCmd>,
    pub local_cmd_receiver: mpsc::Receiver<LocalSwarmCmd>,
    pub network_cmd_receiver: mpsc::Receiver<NetworkSwarmCmd>,
    pub event_sender: mpsc::Sender<NetworkEvent>,
    /// Trackers for underlying behaviour related events
    pub pending_get_closest_peers: PendingGetClosest,
    pub pending_requests:
        HashMap<OutboundRequestId, Option<oneshot::Sender<crate::error::Result<Response>>>>,
    pub pending_get_record: PendingGetRecord,
    /// A list of the most recent peers we have dialed ourselves. Old dialed peers are evicted once the vec fills up.
    pub dialed_peers: CircularVec<PeerId>,
    pub bootstrap_peers: BTreeMap<Option<u32>, HashSet<PeerId>>,
    // Peers that having live connection to. Any peer got contacted during kad network query
    // will have live connection established. And they may not appear in the RT.
    pub live_connected_peers: BTreeMap<ConnectionId, (PeerId, Instant)>,
    // pub known_nodes: BTreeMap<PeerId, Multiaddr>,
    // pub bad_nodes: BadNodes,
    pub network_discovery: NetworkDiscovery,
}

impl SwarmDriverClient {
    pub async fn run(mut self) {
        loop {
            tokio::select! {
                // polls futures in order they appear here (as opposed to random)
                biased;
                // next check if we have locally generated network cmds
                some_cmd = self.network_cmd_receiver.recv() => match some_cmd {
                    Some(cmd) => {
                        let start = Instant::now();
                        let cmd_string = format!("{cmd:?}");
                        if let Err(err) = self.handle_network_cmd(cmd) {
                            warn!("Error while handling cmd: {err}");
                        }
                        trace!("SwarmCmd handled in {:?}: {cmd_string:?}", start.elapsed());
                    },
                    None =>  continue,
                },
                // next take and react to external swarm events
                swarm_event = self.swarm.select_next_some() => {
                    // logging for handling events happens inside handle_swarm_events
                    // otherwise we're rewriting match statements etc around this anwyay
                    if let Err(err) = self.handle_swarm_events(swarm_event) {
                        warn!("Error while handling swarm event: {err}");
                    }
                },
            }
        }
    }

    /// Sends an event after pushing it off thread so as to be non-blocking
    /// this is a wrapper around the `mpsc::Sender::send` call
    pub(crate) fn send_event(&self, event: NetworkEvent) {
        let event_sender = self.event_sender.clone();
        let capacity = event_sender.capacity();

        // push the event off thread so as to be non-blocking
        let _handle = spawn(async move {
            if capacity == 0 {
                warn!(
                    "NetworkEvent channel is full. Await capacity to send: {:?}",
                    event
                );
            }
            if let Err(error) = event_sender.send(event).await {
                error!("SwarmDriver failed to send event: {}", error);
            }
        });
    }

    /// Dials the given multiaddress. If address contains a peer ID, simultaneous
    /// dials to that peer are prevented.
    pub(crate) fn dial(&mut self, mut addr: Multiaddr) -> crate::error::Result<(), DialError> {
        debug!(%addr, "Dialing manually");

        let peer_id = multiaddr_pop_p2p(&mut addr);
        let opts = match peer_id {
            Some(peer_id) => DialOpts::peer_id(peer_id)
                // If we have a peer ID, we can prevent simultaneous dials.
                .condition(PeerCondition::NotDialing)
                .addresses(vec![addr])
                .build(),
            None => DialOpts::unknown_peer_id().address(addr).build(),
        };

        self.swarm.dial(opts)
    }
}
