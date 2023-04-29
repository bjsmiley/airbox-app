use std::{
    collections::HashSet,
    net::{IpAddr, Ipv4Addr},
};

use futures::StreamExt;
use if_watch::{tokio::IfWatcher, IfEvent};

pub struct LanManager {
    pub(crate) lan: HashSet<Ipv4Addr>,
    watch: IfWatcher,
}

impl LanManager {
    pub fn new() -> Result<Self, std::io::Error> {
        let watch = IfWatcher::new()?;
        let mut lan = HashSet::new();
        for net in watch.iter() {
            if let IpAddr::V4(ip) = net.addr() {
                if ip != Ipv4Addr::LOCALHOST {
                    lan.insert(ip);
                }
            }
        }
        Ok(Self { watch, lan })
    }

    pub async fn next(&mut self) -> Result<IfEvent, std::io::Error> {
        self.watch.select_next_some().await
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
