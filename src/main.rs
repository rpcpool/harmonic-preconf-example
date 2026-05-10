// minimal preconf client: authenticate with an ed25519 keypair, subscribe to
// the preconf stream, print each message as it arrives.

use std::path::PathBuf;
use std::time::Duration;

use clap::Parser;
use log::{info, warn};
use solana_keypair::read_keypair_file;
use solana_signer::Signer;
use tonic::metadata::MetadataValue;
use tonic::transport::Endpoint;
use tonic::Request;

pub mod preconf_proto {
    tonic::include_proto!("preconf");
}

use preconf_proto::{
    preconf_auth_service_client::PreconfAuthServiceClient,
    preconf_service_client::PreconfServiceClient, GenerateAuthChallengeRequest,
    GenerateAuthTokensRequest, SubscribePreconfsRequest,
};

#[derive(Parser)]
#[command(about = "subscribe to a preconf stream and print every message")]
struct Cli {
    /// preconf gRPC server URL (e.g. https://preconf.example.com:443)
    #[arg(long)]
    url: String,

    /// ed25519 keypair json, must be authenticated to connect to the preconf server
    #[arg(long)]
    keypair: PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // tonic 0.14 uses rustls 0.23, which has no implicit default crypto
    // provider — install one process-wide before any tls handshake fires.
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("install rustls aws-lc-rs crypto provider");

    let cli = Cli::parse();

    let keypair = read_keypair_file(&cli.keypair)
        .map_err(|e| format!("read keypair {:?}: {}", cli.keypair, e))?;
    let pubkey = keypair.pubkey();
    info!("loaded keypair {}", pubkey);

    // separate channels for auth and stream, so a long-lived stream wont't
    // interfere with token refreshes.
    let auth_channel = build_endpoint(&cli.url)?.connect().await?;
    let stream_channel = build_endpoint(&cli.url)?.connect().await?;

    let mut auth_client = PreconfAuthServiceClient::new(auth_channel);

    let challenge = auth_client
        .generate_auth_challenge(GenerateAuthChallengeRequest {
            pubkey: pubkey.as_ref().to_vec().into(),
        })
        .await?
        .into_inner()
        .challenge;

    let signature = keypair.sign_message(&challenge).as_ref().to_vec();

    let access_token = auth_client
        .generate_auth_tokens(GenerateAuthTokensRequest {
            pubkey: pubkey.as_ref().to_vec().into(),
            challenge: challenge.clone(),
            signature: signature.into(),
        })
        .await?
        .into_inner()
        .access_token
        .ok_or("auth response missing access_token")?;

    info!(
        "authenticated, token expires_at_ms={}",
        access_token.expires_at_ms
    );

    let bearer: MetadataValue<_> = format!("Bearer {}", access_token.value).parse()?;

    let mut stream_client =
        PreconfServiceClient::with_interceptor(stream_channel, move |mut req: Request<()>| {
            req.metadata_mut().insert("authorization", bearer.clone());
            Ok(req)
        });

    let mut stream = stream_client
        .subscribe_preconfs(SubscribePreconfsRequest {})
        .await?
        .into_inner();

    info!("subscribed, streaming...");

    while let Some(msg) = stream.message().await? {
        println!("slot={} txn_bytes={}", msg.slot, msg.data.len());
    }

    warn!("stream ended");
    Ok(())
}

fn build_endpoint(url: &str) -> Result<Endpoint, Box<dyn std::error::Error>> {
    let mut ep = Endpoint::from_shared(url.to_string())?
        .http2_keep_alive_interval(Duration::from_secs(10))
        .keep_alive_timeout(Duration::from_secs(5))
        .http2_adaptive_window(true)
        .initial_connection_window_size(16 * 1024 * 1024)
        .initial_stream_window_size(4 * 1024 * 1024);

    if url.starts_with("https://") {
        let tls = tonic::transport::ClientTlsConfig::new().with_native_roots();
        ep = ep.tls_config(tls)?;
    }
    Ok(ep)
}
