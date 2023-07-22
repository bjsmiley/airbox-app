using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Text.Json.Serialization;
using System.Text.Json;
using System.Threading.Tasks;

namespace FlyDrop.Core.Models.Commands
{
    [JsonConverter(typeof(StopDiscoveryRequestJsonConverter))]
    public class StopDiscoveryRequest : CommandRequest
    {
    }

    public class StopDiscoveryResponse : CommandResponse
    {

    }

    internal class StopDiscoveryRequestJsonConverter : JsonConverter<StopDiscoveryRequest>
    {
        public override StopDiscoveryRequest? Read(ref Utf8JsonReader reader, Type typeToConvert, JsonSerializerOptions options)
        {
            throw new NotImplementedException();
        }

        public override void Write(Utf8JsonWriter writer, StopDiscoveryRequest value, JsonSerializerOptions options)
        {
            writer.WriteRawValue("\"StopDiscovery\"");
        }
    }
}
