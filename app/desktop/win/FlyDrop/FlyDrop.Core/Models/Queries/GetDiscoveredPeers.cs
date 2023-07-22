using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Text.Json.Serialization;
using System.Text.Json;
using System.Threading.Tasks;
using FlyDrop.Core.Models.Common;

namespace FlyDrop.Core.Models.Queries
{
    [JsonConverter(typeof(GetDiscoveredPeersRequestJsonConverter))]
    public class GetDiscoveredPeersRequest: QueryRequest { }

    public class GetDiscoveredPeersResponse: QueryResponse 
    {
        public List<PeerMetadata> DiscoveredPeers { get; set; }

    }

    internal class GetDiscoveredPeersRequestJsonConverter : JsonConverter<GetDiscoveredPeersRequest>
    {
        public override GetDiscoveredPeersRequest? Read(ref Utf8JsonReader reader, Type typeToConvert, JsonSerializerOptions options)
        {
            throw new NotImplementedException();
        }

        public override void Write(Utf8JsonWriter writer, GetDiscoveredPeersRequest value, JsonSerializerOptions options)
        {
            writer.WriteRawValue("\"GetDiscoveredPeers\"");
        }
    }
}
