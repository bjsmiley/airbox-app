using FlyDrop.Core.Models.Queries;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Text.Json.Serialization;
using System.Text.Json;
using System.Threading.Tasks;

namespace FlyDrop.Core.Models.Commands
{
    [JsonConverter(typeof(StartDiscoveryRequestJsonConverter))]
    public class StartDiscoveryRequest: CommandRequest
    {
    }

    public class StartDiscoveryResponse: CommandResponse
    {

    }

    internal class StartDiscoveryRequestJsonConverter : JsonConverter<StartDiscoveryRequest>
    {
        public override StartDiscoveryRequest? Read(ref Utf8JsonReader reader, Type typeToConvert, JsonSerializerOptions options)
        {
            throw new NotImplementedException();
        }

        public override void Write(Utf8JsonWriter writer, StartDiscoveryRequest value, JsonSerializerOptions options)
        {
            writer.WriteRawValue("\"StartDiscovery\"");
        }
    }
}
