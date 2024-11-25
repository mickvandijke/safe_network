// Copyright 2024 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::client::driver::SwarmDriverClient;
use crate::{
    driver::PendingGetClosestType,
    error::{NetworkError, Result},
    multiaddr_pop_p2p, GetRecordCfg, GetRecordError, MsgResponder, NetworkEvent,
};
use libp2p::{
    kad::{Quorum, Record, RecordKey},
    Multiaddr, PeerId,
};
use sn_evm::{AttoTokens, PaymentQuote, QuotingMetrics};
use sn_protocol::{
    messages::{Request, Response},
    storage::RecordType,
    NetworkAddress, PrettyPrintRecordKey,
};
use std::{
    collections::{BTreeMap, HashMap},
    fmt::Debug,
};
use tokio::sync::oneshot;

#[derive(Debug, Eq, PartialEq)]
pub enum NodeIssue {
    /// Data Replication failed
    ReplicationFailure,
    /// Close nodes have reported this peer as bad
    CloseNodesShunning,
    /// Provided a bad quote
    BadQuoting,
    /// Peer failed to pass the chunk proof verification
    FailedChunkProofCheck,
}

/// Commands to send to the Swarm
pub enum LocalSwarmCmd {
    /// Get a map where each key is the ilog2 distance of that Kbucket and each value is a vector of peers in that
    /// bucket.
    GetKBuckets {
        sender: oneshot::Sender<BTreeMap<u32, Vec<PeerId>>>,
    },
    // Returns up to K_VALUE peers from all the k-buckets from the local Routing Table.
    // And our PeerId as well.
    GetClosestKLocalPeers {
        sender: oneshot::Sender<Vec<PeerId>>,
    },
    // Get closest peers from the local RoutingTable
    GetCloseGroupLocalPeers {
        key: NetworkAddress,
        sender: oneshot::Sender<Vec<PeerId>>,
    },
    GetSwarmLocalState(oneshot::Sender<SwarmLocalState>),
    /// Check if the local RecordStore contains the provided key
    RecordStoreHasKey {
        key: RecordKey,
        sender: oneshot::Sender<bool>,
    },
    /// Get the Addresses of all the Records held locally
    GetAllLocalRecordAddresses {
        sender: oneshot::Sender<HashMap<NetworkAddress, RecordType>>,
    },
    /// Get data from the local RecordStore
    GetLocalRecord {
        key: RecordKey,
        sender: oneshot::Sender<Option<Record>>,
    },
    /// GetLocalStoreCost for this node, also with the bad_node list close to the target
    GetLocalStoreCost {
        key: RecordKey,
        sender: oneshot::Sender<(AttoTokens, QuotingMetrics, Vec<NetworkAddress>)>,
    },
    /// Notify the node received a payment.
    PaymentReceived,
    /// Put record to the local RecordStore
    PutLocalRecord {
        record: Record,
    },
    /// Remove a local record from the RecordStore
    /// Typically because the write failed
    RemoveFailedLocalRecord {
        key: RecordKey,
    },
    /// Add a local record to the RecordStore's HashSet of stored records
    /// This should be done after the record has been stored to disk
    AddLocalRecordAsStored {
        key: RecordKey,
        record_type: RecordType,
    },
    /// Add a peer to the blocklist
    AddPeerToBlockList {
        peer_id: PeerId,
    },
    /// Notify whether peer is in trouble
    RecordNodeIssue {
        peer_id: PeerId,
        issue: NodeIssue,
    },
    // Whether peer is considered as `in trouble` by self
    IsPeerShunned {
        target: NetworkAddress,
        sender: oneshot::Sender<bool>,
    },
    // Quote verification agaisnt historical collected quotes
    QuoteVerification {
        quotes: Vec<(PeerId, PaymentQuote)>,
    },
    // Notify a fetch completion
    FetchCompleted((RecordKey, RecordType)),
    /// Triggers interval repliation
    /// NOTE: This does result in outgoing messages, but is produced locally
    TriggerIntervalReplication,
    /// Triggers unrelevant record cleanup
    TriggerIrrelevantRecordCleanup,
}

/// Commands to send to the Swarm
pub enum NetworkSwarmCmd {
    Dial {
        addr: Multiaddr,
        sender: oneshot::Sender<Result<()>>,
    },
    // Get closest peers from the network
    GetClosestPeersToAddressFromNetwork {
        key: NetworkAddress,
        sender: oneshot::Sender<Vec<PeerId>>,
    },

    // Send Request to the PeerId.
    SendRequest {
        req: Request,
        peer: PeerId,

        // If a `sender` is provided, the requesting node will await for a `Response` from the
        // Peer. The result is then returned at the call site.
        //
        // If a `sender` is not provided, the requesting node will not wait for the Peer's
        // response. Instead we trigger a `NetworkEvent::ResponseReceived` which calls the common
        // `response_handler`
        sender: Option<oneshot::Sender<Result<Response>>>,
    },
    SendResponse {
        resp: Response,
        channel: MsgResponder,
    },

    /// Get Record from the Kad network
    GetNetworkRecord {
        key: RecordKey,
        sender: oneshot::Sender<std::result::Result<Record, GetRecordError>>,
        cfg: GetRecordCfg,
    },

    /// Put record to network
    PutRecord {
        record: Record,
        sender: oneshot::Sender<Result<()>>,
        quorum: Quorum,
    },
    /// Put record to specific node
    PutRecordTo {
        peers: Vec<PeerId>,
        record: Record,
        sender: oneshot::Sender<Result<()>>,
        quorum: Quorum,
    },
}

/// Debug impl for LocalSwarmCmd to avoid printing full Record, instead only RecodKey
/// and RecordKind are printed.
impl Debug for LocalSwarmCmd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LocalSwarmCmd::PutLocalRecord { record } => {
                write!(
                    f,
                    "LocalSwarmCmd::PutLocalRecord {{ key: {:?} }}",
                    PrettyPrintRecordKey::from(&record.key)
                )
            }
            LocalSwarmCmd::RemoveFailedLocalRecord { key } => {
                write!(
                    f,
                    "LocalSwarmCmd::RemoveFailedLocalRecord {{ key: {:?} }}",
                    PrettyPrintRecordKey::from(key)
                )
            }
            LocalSwarmCmd::AddLocalRecordAsStored { key, record_type } => {
                write!(
                    f,
                    "LocalSwarmCmd::AddLocalRecordAsStored {{ key: {:?}, record_type: {record_type:?} }}",
                    PrettyPrintRecordKey::from(key)
                )
            }

            LocalSwarmCmd::GetClosestKLocalPeers { .. } => {
                write!(f, "LocalSwarmCmd::GetClosestKLocalPeers")
            }
            LocalSwarmCmd::GetCloseGroupLocalPeers { key, .. } => {
                write!(
                    f,
                    "LocalSwarmCmd::GetCloseGroupLocalPeers {{ key: {key:?} }}"
                )
            }
            LocalSwarmCmd::GetLocalStoreCost { .. } => {
                write!(f, "LocalSwarmCmd::GetLocalStoreCost")
            }
            LocalSwarmCmd::PaymentReceived => {
                write!(f, "LocalSwarmCmd::PaymentReceived")
            }
            LocalSwarmCmd::GetLocalRecord { key, .. } => {
                write!(
                    f,
                    "LocalSwarmCmd::GetLocalRecord {{ key: {:?} }}",
                    PrettyPrintRecordKey::from(key)
                )
            }
            LocalSwarmCmd::GetAllLocalRecordAddresses { .. } => {
                write!(f, "LocalSwarmCmd::GetAllLocalRecordAddresses")
            }
            LocalSwarmCmd::GetKBuckets { .. } => {
                write!(f, "LocalSwarmCmd::GetKBuckets")
            }
            LocalSwarmCmd::GetSwarmLocalState { .. } => {
                write!(f, "LocalSwarmCmd::GetSwarmLocalState")
            }
            LocalSwarmCmd::RecordStoreHasKey { key, .. } => {
                write!(
                    f,
                    "LocalSwarmCmd::RecordStoreHasKey {:?}",
                    PrettyPrintRecordKey::from(key)
                )
            }
            LocalSwarmCmd::AddPeerToBlockList { peer_id } => {
                write!(f, "LocalSwarmCmd::AddPeerToBlockList {peer_id:?}")
            }
            LocalSwarmCmd::RecordNodeIssue { peer_id, issue } => {
                write!(
                    f,
                    "LocalSwarmCmd::SendNodeStatus peer {peer_id:?}, issue: {issue:?}"
                )
            }
            LocalSwarmCmd::IsPeerShunned { target, .. } => {
                write!(f, "LocalSwarmCmd::IsPeerInTrouble target: {target:?}")
            }
            LocalSwarmCmd::QuoteVerification { quotes } => {
                write!(
                    f,
                    "LocalSwarmCmd::QuoteVerification of {} quotes",
                    quotes.len()
                )
            }
            LocalSwarmCmd::FetchCompleted((key, record_type)) => {
                write!(
                    f,
                    "LocalSwarmCmd::FetchCompleted({record_type:?} : {:?})",
                    PrettyPrintRecordKey::from(key)
                )
            }
            LocalSwarmCmd::TriggerIntervalReplication => {
                write!(f, "LocalSwarmCmd::TriggerIntervalReplication")
            }
            LocalSwarmCmd::TriggerIrrelevantRecordCleanup => {
                write!(f, "LocalSwarmCmd::TriggerUnrelevantRecordCleanup")
            }
        }
    }
}

/// Debug impl for NetworkSwarmCmd to avoid printing full Record, instead only RecodKey
/// and RecordKind are printed.
impl Debug for NetworkSwarmCmd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NetworkSwarmCmd::Dial { addr, .. } => {
                write!(f, "NetworkSwarmCmd::Dial {{ addr: {addr:?} }}")
            }
            NetworkSwarmCmd::GetNetworkRecord { key, cfg, .. } => {
                write!(
                    f,
                    "NetworkSwarmCmd::GetNetworkRecord {{ key: {:?}, cfg: {cfg:?}",
                    PrettyPrintRecordKey::from(key)
                )
            }
            NetworkSwarmCmd::PutRecord { record, .. } => {
                write!(
                    f,
                    "NetworkSwarmCmd::PutRecord {{ key: {:?} }}",
                    PrettyPrintRecordKey::from(&record.key)
                )
            }
            NetworkSwarmCmd::PutRecordTo { peers, record, .. } => {
                write!(
                    f,
                    "NetworkSwarmCmd::PutRecordTo {{ peers: {peers:?}, key: {:?} }}",
                    PrettyPrintRecordKey::from(&record.key)
                )
            }
            NetworkSwarmCmd::GetClosestPeersToAddressFromNetwork { key, .. } => {
                write!(f, "NetworkSwarmCmd::GetClosestPeers {{ key: {key:?} }}")
            }
            NetworkSwarmCmd::SendResponse { resp, .. } => {
                write!(f, "NetworkSwarmCmd::SendResponse resp: {resp:?}")
            }
            NetworkSwarmCmd::SendRequest { req, peer, .. } => {
                write!(
                    f,
                    "NetworkSwarmCmd::SendRequest req: {req:?}, peer: {peer:?}"
                )
            }
        }
    }
}
/// Snapshot of information kept in the Swarm's local state
#[derive(Debug, Clone)]
pub struct SwarmLocalState {
    /// List of currently connected peers
    pub connected_peers: Vec<PeerId>,
    /// List of addresses the node is currently listening on
    pub listeners: Vec<Multiaddr>,
}

impl SwarmDriverClient {
    pub(crate) fn handle_network_cmd(&mut self, cmd: NetworkSwarmCmd) -> Result<(), NetworkError> {
        match cmd {
            NetworkSwarmCmd::GetNetworkRecord { key, sender, cfg } => {
                for (pending_query, (inflight_record_query_key, senders, _, _)) in
                    self.pending_get_record.iter_mut()
                {
                    if *inflight_record_query_key == key {
                        debug!(
                            "GetNetworkRecord for {:?} is already in progress. Adding sender to {pending_query:?}",
                            PrettyPrintRecordKey::from(&key)
                        );
                        senders.push(sender);

                        // early exit as we're already processing this query
                        return Ok(());
                    }
                }

                let query_id = self.swarm.behaviour_mut().kademlia.get_record(key.clone());

                debug!(
                    "Record {:?} with task {query_id:?} expected to be held by {:?}",
                    PrettyPrintRecordKey::from(&key),
                    cfg.expected_holders
                );

                if self
                    .pending_get_record
                    .insert(query_id, (key, vec![sender], Default::default(), cfg))
                    .is_some()
                {
                    warn!("An existing get_record task {query_id:?} got replaced");
                }
                // Logging the status of the `pending_get_record`.
                // We also interested in the status of `result_map` (which contains record) inside.
                let total_records: usize = self
                    .pending_get_record
                    .iter()
                    .map(|(_, (_, _, result_map, _))| result_map.len())
                    .sum();
                info!("We now have {} pending get record attempts and cached {total_records} fetched copies",
                      self.pending_get_record.len());
            }
            NetworkSwarmCmd::PutRecord {
                record,
                sender,
                quorum,
            } => {
                let record_key = PrettyPrintRecordKey::from(&record.key).into_owned();
                debug!(
                    "Putting record sized: {:?} to network {:?}",
                    record.value.len(),
                    record_key
                );
                let res = match self
                    .swarm
                    .behaviour_mut()
                    .kademlia
                    .put_record(record, quorum)
                {
                    Ok(request_id) => {
                        debug!("Sent record {record_key:?} to network. Request id: {request_id:?} to network");
                        Ok(())
                    }
                    Err(error) => {
                        error!("Error sending record {record_key:?} to network");
                        Err(NetworkError::from(error))
                    }
                };

                if let Err(err) = sender.send(res) {
                    error!("Could not send response to PutRecord cmd: {:?}", err);
                }
            }
            NetworkSwarmCmd::PutRecordTo {
                peers,
                record,
                sender,
                quorum,
            } => {
                let record_key = PrettyPrintRecordKey::from(&record.key).into_owned();
                debug!(
                    "Putting record {record_key:?} sized: {:?} to {peers:?}",
                    record.value.len(),
                );
                let peers_count = peers.len();
                let request_id = self.swarm.behaviour_mut().kademlia.put_record_to(
                    record,
                    peers.into_iter(),
                    quorum,
                );
                debug!("Sent record {record_key:?} to {peers_count:?} peers. Request id: {request_id:?}");

                if let Err(err) = sender.send(Ok(())) {
                    error!("Could not send response to PutRecordTo cmd: {:?}", err);
                }
            }
            NetworkSwarmCmd::Dial { addr, sender } => {
                if let Some(peer_id) = multiaddr_pop_p2p(&mut addr.clone()) {
                    // Only consider the dial peer is bootstrap node when proper PeerId is provided.
                    if let Some(kbucket) = self.swarm.behaviour_mut().kademlia.kbucket(peer_id) {
                        let ilog2 = kbucket.range().0.ilog2();
                        let peers = self.bootstrap_peers.entry(ilog2).or_default();
                        peers.insert(peer_id);
                    }
                }
                let _ = match self.dial(addr) {
                    Ok(_) => sender.send(Ok(())),
                    Err(e) => sender.send(Err(e.into())),
                };
            }
            NetworkSwarmCmd::GetClosestPeersToAddressFromNetwork { key, sender } => {
                let query_id = self
                    .swarm
                    .behaviour_mut()
                    .kademlia
                    .get_closest_peers(key.as_bytes());
                let _ = self.pending_get_closest_peers.insert(
                    query_id,
                    (
                        PendingGetClosestType::FunctionCall(sender),
                        Default::default(),
                    ),
                );
            }
            NetworkSwarmCmd::SendRequest { req, peer, sender } => {
                // If `self` is the recipient, forward the request directly to our upper layer to
                // be handled.
                // `self` then handles the request and sends a response back again to itself.
                if peer == *self.swarm.local_peer_id() {
                    trace!("Sending query request to self");
                    if let Request::Query(query) = req {
                        self.send_event(NetworkEvent::QueryRequestReceived {
                            query,
                            channel: MsgResponder::FromSelf(sender),
                        });
                    } else {
                        // We should never receive a Replicate request from ourselves.
                        // we already hold this data if we do... so we can ignore
                        trace!("Replicate cmd to self received, ignoring");
                    }
                } else {
                    let request_id = self
                        .swarm
                        .behaviour_mut()
                        .request_response
                        .send_request(&peer, req);
                    trace!("Sending request {request_id:?} to peer {peer:?}");
                    let _ = self.pending_requests.insert(request_id, sender);

                    trace!("Pending Requests now: {:?}", self.pending_requests.len());
                }
            }
            _ => {}
        }

        Ok(())
    }
}
