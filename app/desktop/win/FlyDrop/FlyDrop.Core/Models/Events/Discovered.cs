using FlyDrop.Core.Models.Common;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Text.Json;
using System.Text.Json.Serialization;
using System.Threading.Tasks;

namespace FlyDrop.Core.Models.Events
{
    [JsonConverter(typeof(DiscoveredEventJsonConverter))]
    public class DiscoveredEvent: ApiEvent
    {
        public PeerMetadata Discovered { get; set; }
    }

    public class DiscoveredEventJsonConverter : JsonConverter<DiscoveredEvent>
    {
        public override DiscoveredEvent? Read(ref Utf8JsonReader reader, Type typeToConvert, JsonSerializerOptions options)
        {
            return new DiscoveredEvent
            {
                Discovered = JsonSerializer.Deserialize<PeerMetadata>(ref reader, options)
            };
        }

        public override void Write(Utf8JsonWriter writer, DiscoveredEvent value, JsonSerializerOptions options)
        {
            throw new NotImplementedException();
        }
    }
}
