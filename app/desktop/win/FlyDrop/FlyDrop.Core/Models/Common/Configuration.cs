using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Text.Json.Serialization;
using System.Threading.Tasks;

namespace FlyDrop.Core.Models.Common
{
    public class Configuration
    {
        // [JsonPropertyName("name")]
        public string Name { get; set; }
        // [JsonPropertyName("id")]
        public string Id { get; set; }

        // [JsonPropertyName("known_peers")]
        public List<PeerMetadata> KnownPeers { get; set; }
       //  [JsonPropertyName("auto_accept")]
        public bool AutoAccept { get; set; }

    }
}
