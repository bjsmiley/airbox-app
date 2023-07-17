use std::{
    collections::HashSet,
    net::{IpAddr, Ipv4Addr},
};

use futures::StreamExt;
use if_watch::{tokio::IfWatcher, IfEvent, IpNet, Ipv4Net};

pub struct LanManager {
    pub(crate) lan: HashSet<Ipv4Addr>,
    watch: IfWatcher,
}

impl LanManager {
    pub fn new() -> Result<Self, std::io::Error> {
        let watch = IfWatcher::new()?;
        let lan = HashSet::new();
        // for net in watch.iter() {
        //     if let IpAddr::V4(ip) = net.addr() {
        //         if ip != Ipv4Addr::LOCALHOST {
        //             lan.insert(ip);
        //         }
        //     }
        // }
        Ok(Self { lan, watch })
    }

    pub async fn next(&mut self) -> Result<IfEvent, std::io::Error> {
        self.watch.select_next_some().await
    }

    pub async fn next_ipv4_up(&mut self) -> Ipv4Addr {
        loop {
            if let Ok(IfEvent::Up(IpNet::V4(ipv4))) = self.next().await {
                let addr = ipv4.addr();
                if addr != Ipv4Addr::LOCALHOST {
                    return addr;
                }
            }
        }
    }
}

// pub fn lan_ips() -> Result<Vec<Ipv4Addr>, std::io::Error> {
//     let set = IfWatcher::new()?;
//     let mut output = HashSet::new();
//     for net in set.iter() {
//         if let IpAddr::V4(ip) = net.addr() {
//             if ip != Ipv4Addr::LOCALHOST {
//                 output.insert(ip);
//             }
//         }
//     }
//     return Ok(output);
// }
