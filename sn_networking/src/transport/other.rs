#[cfg(feature = "open-metrics")]
use crate::MetricsRegistries;
#[cfg(feature = "webrtc")]
use futures::future::Either;
use libp2p::{
    core::{muxing::StreamMuxerBox, transport},
    identity::Keypair,
    PeerId, Transport as _,
};
use libp2p_webrtc as webrtc;
use rand::thread_rng;

pub(crate) fn build_transport(
    keypair: &Keypair,
    #[cfg(feature = "open-metrics")] registries: &mut MetricsRegistries,
) -> transport::Boxed<(PeerId, StreamMuxerBox)> {
    let trans = generate_quic_transport(keypair);
    #[cfg(feature = "open-metrics")]
    let trans = libp2p::metrics::BandwidthTransport::new(trans, &mut registries.standard_metrics);

    #[cfg(feature = "webrtc")]
    let generate_webrtc_transport = webrtc::tokio::Transport::new(
        keypair.clone(),
        webrtc::tokio::Certificate::generate(&mut thread_rng())
            .expect("Could not generate certificate for WebRTC"),
    );

    #[cfg(feature = "webrtc")]
    let trans = trans
        .or_transport(generate_webrtc_transport)
        .map(|either_output, _| match either_output {
            Either::Left((peer_id, muxer)) => (peer_id, StreamMuxerBox::new(muxer)),
            Either::Right((peer_id, muxer)) => (peer_id, StreamMuxerBox::new(muxer)),
        });

    #[cfg(not(feature = "webrtc"))]
    let trans = trans.map(|(peer_id, muxer), _| (peer_id, StreamMuxerBox::new(muxer)));

    trans.boxed()
}

fn generate_quic_transport(
    keypair: &Keypair,
) -> libp2p::quic::GenTransport<libp2p::quic::tokio::Provider> {
    libp2p::quic::tokio::Transport::new(libp2p::quic::Config::new(keypair))
}
