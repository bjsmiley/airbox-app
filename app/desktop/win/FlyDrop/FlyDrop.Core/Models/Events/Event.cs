using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Text.Json;
using System.Text.Json.Serialization;
using System.Threading.Tasks;

namespace FlyDrop.Core.Models.Events
{

    [JsonConverter(typeof(ApiEventJsonConverter))]
    public abstract class ApiEvent
    {
    }

    internal class ApiEventJsonConverter : JsonConverter<ApiEvent>
    {
        public override ApiEvent? Read(ref Utf8JsonReader reader, Type typeToConvert, JsonSerializerOptions options)
        {
            if(reader.TokenType == JsonTokenType.StartObject && reader.Read() && reader.TokenType == JsonTokenType.PropertyName)
            {
                var name = reader.GetString();
                ApiEvent? content = name switch
                {
                    "Discovered" => JsonSerializer.Deserialize<DiscoveredEvent>(ref reader, options),
                    "AppControl" => JsonSerializer.Deserialize<AppControlEvent<ControlContent>>(ref reader, options),
                    "AppControlUpdate" => JsonSerializer.Deserialize<AppControlUpdateEvent>(ref reader, options),
                    _ => throw new InvalidOperationException()
                };
                reader.Read();
                return content;
            }
            throw new JsonException();
        }

        public override void Write(Utf8JsonWriter writer, ApiEvent value, JsonSerializerOptions options)
        {
            throw new NotImplementedException();
        }
    }
}
