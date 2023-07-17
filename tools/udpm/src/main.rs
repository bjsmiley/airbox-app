use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use clap::{Parser, Subcommand};
use futures::{SinkExt, StreamExt};
use p2p::{event::DiscoveryEvent, peer::PeerMetadata};
use tokio::net::UdpSocket;
use tokio_util::udp::UdpFramed;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[arg(short, long, default_value_t = p2p::discovery::DISCOVERY_MULTICAST)]
    multicast_address: Ipv4Addr,
    #[arg(short, long, default_value_t = 50692)]
    port: u16,
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
pub enum Cmd {
    Serve,
    Send {
        #[arg(short, long)]
        msg_type: Option<String>,
    },
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let cli = Cli::parse();

    let (sock, addr) = p2p::discovery::multicast(
        &SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), cli.port),
        &SocketAddr::new(IpAddr::V4(cli.multicast_address), cli.port),
    )?;

    match cli.command {
        Cmd::Serve => serve(sock).await,
        Cmd::Send { msg_type } => send(sock, addr, msg_type).await,
    }
}

async fn serve(sock: UdpSocket) -> std::io::Result<()> {
    let mut f = UdpFramed::new(sock, p2p::proto::DiscoveryCodec);
    let mut i = 0;
    loop {
        match f.next().await {
            None => return Ok(()),
            Some(result) => {
                let frame = result.unwrap();
                println!("[{}] recv: {:?} from {}", i, frame.0, frame.1);
                i += 1;
            }
        }
    }
}

async fn send(sock: UdpSocket, addr: SocketAddr, msg_type: Option<String>) -> std::io::Result<()> {
    let mut f = UdpFramed::new(sock, p2p::proto::DiscoveryCodec);
    let e = match msg_type.as_deref() {
        Some("response") => {
            let mut meta = PeerMetadata::default();
            meta.name = String::from("udpm-cli");
            DiscoveryEvent::PresenceResponse(meta)
        }
        _ => DiscoveryEvent::PresenceRequest(0),
    };
    f.send((e, addr)).await.unwrap();
    Ok(())
}
