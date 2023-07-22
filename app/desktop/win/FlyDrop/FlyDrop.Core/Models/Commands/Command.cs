using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Text.Json;
using System.Text.Json.Serialization;
using System.Threading.Tasks;

namespace FlyDrop.Core.Models.Commands
{
    public abstract class CommandRequest
    {
    }

    [JsonConverter(typeof(CommandResponseJsonConverter))]
    public class CommandResponse
    {
    }

    public class CommandResponseJsonConverter : JsonConverter<CommandResponse>
    {
        public override CommandResponse? Read(ref Utf8JsonReader reader, Type typeToConvert, JsonSerializerOptions options)
        {
            if (reader.GetString() == "Ok")
                return new CommandResponse();
            throw new JsonException();
        }

        public override void Write(Utf8JsonWriter writer, CommandResponse value, JsonSerializerOptions options)
        {
            throw new NotImplementedException();
        }
    }
}
