use crate::client::driver::SwarmDriverClient;
use crate::event::NodeEvent;
use crate::relay_manager::is_a_relayed_peer;
use crate::{multiaddr_is_global, multiaddr_strip_p2p, NetworkEvent};
use libp2p::kad::K_VALUE;
#[cfg(feature = "local")]
use libp2p::mdns;
use libp2p::multiaddr::Protocol;
use libp2p::swarm::dial_opts::{DialOpts, PeerCondition};
use libp2p::swarm::{DialError, SwarmEvent};
use libp2p::{Multiaddr, TransportError};
use sn_protocol::version::{IDENTIFY_NODE_VERSION_STR, IDENTIFY_PROTOCOL_STR};
use std::collections::HashSet;
use std::time::{Duration, Instant};

impl SwarmDriverClient {
    pub(crate) fn handle_swarm_events(
        &mut self,
        event: SwarmEvent<NodeEvent>,
    ) -> crate::error::Result<()> {
        match event {
            SwarmEvent::Behaviour(NodeEvent::Kademlia(kad_event)) => {
                self.handle_kad_event(kad_event)?;
            }
            SwarmEvent::Behaviour(NodeEvent::Identify(iden)) => {
                match *iden {
                    libp2p::identify::Event::Received {
                        peer_id,
                        info,
                        connection_id,
                    } => {
                        debug!(conn_id=%connection_id, %peer_id, ?info, "identify: received info");

                        if info.protocol_version != IDENTIFY_PROTOCOL_STR.to_string() {
                            warn!(?info.protocol_version, "identify: {peer_id:?} does not have the same protocol. Our IDENTIFY_PROTOCOL_STR: {:?}", IDENTIFY_PROTOCOL_STR.as_str());

                            self.send_event(NetworkEvent::PeerWithUnsupportedProtocol {
                                our_protocol: IDENTIFY_PROTOCOL_STR.to_string(),
                                their_protocol: info.protocol_version,
                            });
                            // Block the peer from any further communication.
                            self.swarm.behaviour_mut().blocklist.block_peer(peer_id);
                            if let Some(dead_peer) =
                                self.swarm.behaviour_mut().kademlia.remove_peer(&peer_id)
                            {
                                error!("Clearing out a protocol mistmatch peer from RT. Something went wrong, we should not have added this peer to RT: {peer_id:?}");
                                self.update_on_peer_removal(*dead_peer.node.key.preimage());
                            }

                            return Ok(());
                        }

                        // if client, return.
                        if info.agent_version != IDENTIFY_NODE_VERSION_STR.to_string() {
                            return Ok(());
                        }

                        let has_dialed = self.dialed_peers.contains(&peer_id);

                        // If we're not in local mode, only add globally reachable addresses.
                        // Strip the `/p2p/...` part of the multiaddresses.
                        // Collect into a HashSet directly to avoid multiple allocations and handle deduplication.
                        let mut addrs: HashSet<Multiaddr> = match self.local {
                            true => info
                                .listen_addrs
                                .into_iter()
                                .map(|addr| multiaddr_strip_p2p(&addr))
                                .collect(),
                            false => info
                                .listen_addrs
                                .into_iter()
                                .filter(multiaddr_is_global)
                                .map(|addr| multiaddr_strip_p2p(&addr))
                                .collect(),
                        };

                        let has_relayed = is_a_relayed_peer(&addrs);

                        let is_bootstrap_peer = self
                            .bootstrap_peers
                            .iter()
                            .any(|(_ilog2, peers)| peers.contains(&peer_id));

                        // When received an identify from un-dialed peer, try to dial it
                        // The dial shall trigger the same identify to be sent again and confirm
                        // peer is external accessible, hence safe to be added into RT.
                        if !self.local && !has_dialed {
                            // Only need to dial back for not fulfilled kbucket
                            let (kbucket_full, already_present_in_rt, ilog2) =
                                if let Some(kbucket) =
                                    self.swarm.behaviour_mut().kademlia.kbucket(peer_id)
                                {
                                    let ilog2 = kbucket.range().0.ilog2();
                                    let num_peers = kbucket.num_entries();
                                    let mut is_bucket_full = num_peers >= K_VALUE.into();

                                    // check if peer_id is already a part of RT
                                    let already_present_in_rt = kbucket
                                        .iter()
                                        .any(|entry| entry.node.key.preimage() == &peer_id);

                                    // If the bucket contains any of a bootstrap node,
                                    // consider the bucket is not full and dial back
                                    // so that the bootstrap nodes can be replaced.
                                    if is_bucket_full {
                                        if let Some(peers) = self.bootstrap_peers.get(&ilog2) {
                                            if kbucket.iter().any(|entry| {
                                                peers.contains(entry.node.key.preimage())
                                            }) {
                                                is_bucket_full = false;
                                            }
                                        }
                                    }

                                    (is_bucket_full, already_present_in_rt, ilog2)
                                } else {
                                    return Ok(());
                                };

                            if kbucket_full {
                                debug!("received identify for a full bucket {ilog2:?}, not dialing {peer_id:?} on {addrs:?}");
                                return Ok(());
                            } else if already_present_in_rt {
                                debug!("received identify for {peer_id:?} that is already part of the RT. Not dialing {peer_id:?} on {addrs:?}");
                                return Ok(());
                            }

                            info!(%peer_id, ?addrs, "received identify info from undialed peer for not full kbucket {ilog2:?}, dial back to confirm external accessible");
                            if let Err(err) = self.swarm.dial(
                                DialOpts::peer_id(peer_id)
                                    .condition(PeerCondition::NotDialing)
                                    .addresses(addrs.iter().cloned().collect())
                                    .build(),
                            ) {
                                warn!(%peer_id, ?addrs, "dialing error: {err:?}");
                            }

                            return Ok(());
                        }

                        // If we are not local, we care only for peers that we dialed and thus are reachable.
                        if self.local || has_dialed {
                            // A bad node cannot establish a connection with us. So we can add it to the RT directly.
                            self.remove_bootstrap_from_full(peer_id);

                            // Avoid have `direct link format` addrs co-exists with `relay` addr
                            if has_relayed {
                                addrs.retain(|multiaddr| {
                                    multiaddr.iter().any(|p| matches!(p, Protocol::P2pCircuit))
                                });
                            }

                            debug!(%peer_id, ?addrs, "identify: attempting to add addresses to routing table");

                            // Attempt to add the addresses to the routing table.
                            for multiaddr in addrs {
                                let _routing_update = self
                                    .swarm
                                    .behaviour_mut()
                                    .kademlia
                                    .add_address(&peer_id, multiaddr); // TODO: leads to closure error
                            }
                        }
                    }
                    // Log the other Identify events.
                    libp2p::identify::Event::Sent { .. } => debug!("identify: {iden:?}"),
                    libp2p::identify::Event::Pushed { .. } => debug!("identify: {iden:?}"),
                    libp2p::identify::Event::Error { .. } => debug!("identify: {iden:?}"),
                }
            }
            #[cfg(feature = "local")]
            SwarmEvent::Behaviour(NodeEvent::Mdns(mdns_event)) => {
                match *mdns_event {
                    mdns::Event::Discovered(list) => {
                        if self.local {
                            for (peer_id, addr) in list {
                                // The multiaddr does not contain the peer ID, so add it.
                                let addr = addr.with(Protocol::P2p(peer_id));

                                info!(%addr, "mDNS node discovered and dialing");

                                if let Err(err) = self.dial(addr.clone()) {
                                    warn!(%addr, "mDNS node dial error: {err:?}");
                                }
                            }
                        }
                    }
                    mdns::Event::Expired(peer) => {
                        debug!("mdns peer {peer:?} expired");
                    }
                }
            }

            SwarmEvent::NewListenAddr {
                mut address,
                listener_id,
            } => {
                let local_peer_id = *self.swarm.local_peer_id();
                // Make sure the address ends with `/p2p/<local peer ID>`. In case of relay, `/p2p` is already there.
                if address.iter().last() != Some(Protocol::P2p(local_peer_id)) {
                    address.push(Protocol::P2p(local_peer_id));
                }

                self.send_event(NetworkEvent::NewListenAddr(address.clone()));

                info!("Local node is listening {listener_id:?} on {address:?}");
                println!("Local node is listening on {address:?}"); // TODO: make it print only once
            }
            SwarmEvent::ListenerClosed {
                listener_id,
                addresses,
                reason,
            } => {
                info!("Listener {listener_id:?} with add {addresses:?} has been closed for {reason:?}");
            }
            SwarmEvent::IncomingConnection {
                connection_id,
                local_addr,
                send_back_addr,
            } => {
                debug!("IncomingConnection ({connection_id:?}) with local_addr: {local_addr:?} send_back_addr: {send_back_addr:?}");
            }
            SwarmEvent::ConnectionEstablished {
                peer_id,
                endpoint,
                num_established,
                connection_id,
                concurrent_dial_errors,
                established_in,
            } => {
                debug!(%peer_id, num_established, ?concurrent_dial_errors, "ConnectionEstablished ({connection_id:?}) in {established_in:?}");

                let _ = self.live_connected_peers.insert(
                    connection_id,
                    (peer_id, Instant::now() + Duration::from_secs(60)),
                );
                self.insert_latest_established_connection_ids(
                    connection_id,
                    endpoint.get_remote_address(),
                );
                self.record_connection_metrics();

                if endpoint.is_dialer() {
                    self.dialed_peers.push(peer_id);
                }
            }
            SwarmEvent::ConnectionClosed {
                peer_id,
                endpoint,
                cause,
                num_established,
                connection_id,
            } => {
                debug!(%peer_id, ?connection_id, ?cause, num_established, "ConnectionClosed");
                let _ = self.live_connected_peers.remove(&connection_id);
                self.record_connection_metrics();
            }
            SwarmEvent::OutgoingConnectionError {
                peer_id: Some(failed_peer_id),
                error,
                connection_id,
            } => {
                warn!("OutgoingConnectionError to {failed_peer_id:?} on {connection_id:?} - {error:?}");
                let _ = self.live_connected_peers.remove(&connection_id);
                self.record_connection_metrics();

                // we need to decide if this was a critical error and the peer should be removed from the routing table
                let should_clean_peer = match error {
                    DialError::Transport(errors) => {
                        // as it's an outgoing error, if it's transport based we can assume it is _our_ fault
                        //
                        // (eg, could not get a port for a tcp connection)
                        // so we default to it not being a real issue
                        // unless there are _specific_ errors (connection refused eg)
                        error!("Dial errors len : {:?}", errors.len());
                        let mut there_is_a_serious_issue = false;
                        for (_addr, err) in errors {
                            error!("OutgoingTransport error : {err:?}");

                            match err {
                                TransportError::MultiaddrNotSupported(addr) => {
                                    warn!("Multiaddr not supported : {addr:?}");
                                    #[cfg(feature = "loud")]
                                    {
                                        println!("Multiaddr not supported : {addr:?}");
                                        println!("If this was your bootstrap peer, restart your node with a supported multiaddr");
                                    }
                                    // if we can't dial a peer on a given address, we should remove it from the routing table
                                    there_is_a_serious_issue = true
                                }
                                TransportError::Other(err) => {
                                    let problematic_errors = [
                                        "ConnectionRefused",
                                        "HostUnreachable",
                                        "HandshakeTimedOut",
                                    ];

                                    let is_bootstrap_peer = self
                                        .bootstrap_peers
                                        .iter()
                                        .any(|(_ilog2, peers)| peers.contains(&failed_peer_id));
                                }
                            }
                        }
                        there_is_a_serious_issue
                    }
                    DialError::NoAddresses => {
                        // We provided no address, and while we can't really blame the peer
                        // we also can't connect, so we opt to cleanup...
                        warn!("OutgoingConnectionError: No address provided");
                        true
                    }
                    DialError::Aborted => {
                        // not their fault
                        warn!("OutgoingConnectionError: Aborted");
                        false
                    }
                    DialError::DialPeerConditionFalse(_) => {
                        // we could not dial due to an internal condition, so not their issue
                        warn!("OutgoingConnectionError: DialPeerConditionFalse");
                        false
                    }
                    DialError::LocalPeerId { endpoint, .. } => {
                        // This is actually _us_ So we should remove this from the RT
                        error!("OutgoingConnectionError: LocalPeerId");
                        true
                    }
                    DialError::WrongPeerId { obtained, endpoint } => {
                        // The peer id we attempted to dial was not the one we expected
                        // cleanup
                        error!("OutgoingConnectionError: WrongPeerId: obtained: {obtained:?}, endpoint: {endpoint:?}");
                        true
                    }
                    DialError::Denied { cause } => {
                        // The peer denied our connection
                        // cleanup
                        error!("OutgoingConnectionError: Denied: {cause:?}");
                        true
                    }
                };

                if should_clean_peer {
                    warn!("Tracking issue of {failed_peer_id:?}. Clearing it out for now");

                    if let Some(dead_peer) = self
                        .swarm
                        .behaviour_mut()
                        .kademlia
                        .remove_peer(&failed_peer_id)
                    {
                        self.update_on_peer_removal(*dead_peer.node.key.preimage());
                    }
                }
            }
            SwarmEvent::IncomingConnectionError {
                connection_id,
                local_addr,
                send_back_addr,
                error,
            } => {
                // Only log as ERROR if the the connection is not adjacent to an already established connection id from
                // the same IP address.
                //
                // If a peer contains multiple transports/listen addrs, we might try to open multiple connections,
                // and if the first one passes, we would get error on the rest. We don't want to log these.
                //
                // Also sometimes we get the ConnectionEstablished event immediately after this event.
                // So during tokio::select! of the events, we skip processing IncomingConnectionError for one round,
                // giving time for ConnectionEstablished to be hopefully processed.
                // And since we don't do anything critical with this event, the order and time of processing is
                // not critical.
                if self.should_we_log_incoming_connection_error(connection_id, &send_back_addr) {
                    error!("IncomingConnectionError from local_addr:?{local_addr:?}, send_back_addr {send_back_addr:?} on {connection_id:?} with error {error:?}");
                } else {
                    debug!("IncomingConnectionError from local_addr:?{local_addr:?}, send_back_addr {send_back_addr:?} on {connection_id:?} with error {error:?}");
                }
                let _ = self.live_connected_peers.remove(&connection_id);
                self.record_connection_metrics();
            }
            SwarmEvent::Dialing {
                peer_id,
                connection_id,
            } => {
                debug!("Dialing {peer_id:?} on {connection_id:?}");
            }
            SwarmEvent::ExternalAddrConfirmed { address } => {
                info!(%address, "external address: confirmed");
            }
            SwarmEvent::ExternalAddrExpired { address } => {
                info!(%address, "external address: expired");
            }
            other => {
                debug!("SwarmEvent has been ignored: {other:?}")
            }
        }

        Ok(())
    }
}
