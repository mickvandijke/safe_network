// wasm32 environments typically only support WebSockets (and WebRTC or WebTransport), so no plain UDP or TCP.

use libp2p::{
    core::{muxing::StreamMuxerBox, transport},
    identity::Keypair,
    PeerId,
};

pub(crate) fn build_transport(keypair: &Keypair) -> transport::Boxed<(PeerId, StreamMuxerBox)> {
    libp2p_webrtc_websys::Transport::new(libp2p_webrtc_websys::Config::new(&keypair)).boxed()
}
