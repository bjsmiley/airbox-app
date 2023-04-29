use std::collections::HashSet;
use std::io::Write;
use std::path;

use p2p::peer;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;

use crate::err::ConfError;
use crate::plat;
use crate::secret;

pub static NODE_CONFIG_NAME: &str = "settings.json";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NodeConfig {
    pub name: String,
    #[serde(skip)]
    pub id: peer::PeerId,
    pub known_peers: HashSet<peer::PeerMetadata>,
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            name: plat::host_name(),
            known_peers: HashSet::new(),
            id: peer::PeerId::default(),
        }
    }
}

pub struct NodeConfigStore(String);

impl NodeConfigStore {
    pub fn set(&self, conf: &NodeConfig) -> Result<(), ConfError> {
        // only write to disk if config path is set
        if !self.0.is_empty() {
            let mut builder = path::PathBuf::from(self.0.clone());
            builder.push(NODE_CONFIG_NAME);
            let path = builder.as_path();
            let mut file = fs::File::create(path)?;
            let json = serde_json::to_string(conf)?;
            file.write_all(json.as_bytes())?;
        }
        Ok(())
    }

    pub fn get(&self) -> Result<NodeConfig, ConfError> {
        let mut conf = self
            .from_disk()
            .or_else(|_| -> Result<NodeConfig, ConfError> { Ok(NodeConfig::default()) })?;
        let (cert, _) = secret::get_identity()?.into_rustls();
        conf.id = peer::PeerId::from_cert(&cert);
        Ok(conf)
    }

    fn from_disk(&self) -> Result<NodeConfig, ConfError> {
        let mut builder = path::PathBuf::from(self.0.clone());
        builder.push(NODE_CONFIG_NAME);
        let path = builder.as_path();
        let file = fs::File::open(path)?;
        let reader = io::BufReader::new(file);
        let config = serde_json::from_reader(reader)?;
        Ok(config)
    }
}

impl From<String> for NodeConfigStore {
    fn from(value: String) -> Self {
        Self(value)
    }
}

#[cfg(test)]
mod tests {

    use p2p::peer::PeerId;

    use crate::conf::{NodeConfigStore, NODE_CONFIG_NAME};
    use crate::err::ConfError;
    use crate::secret::mock_store;

    #[test]
    pub fn get_set_conf() -> Result<(), ConfError> {
        mock_store();
        let dir = String::from("C:\\Users\\bryan\\AppData\\Local\\Temp"); // TODO
        let store = NodeConfigStore(dir.clone());
        let mut conf = store.get()?;
        assert_ne!(PeerId::default(), conf.id);
        conf.name = String::from("override name");
        store.set(&conf)?;
        let conf = store.get()?;
        assert_eq!("override name", conf.name);
        // cleanup
        let path = std::path::Path::new(&dir).join(NODE_CONFIG_NAME);
        _ = std::fs::remove_file(path);
        Ok(())
    }
}
