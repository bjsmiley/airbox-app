using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Text.Json;
using System.Text.Json.Serialization;
using System.Threading.Tasks;

namespace FlyDrop.Core.Models.Events
{
    internal class AppControlEvent<T>: ApiEvent where T: ControlContent
    {
        public string Peer { get; set; }
        public ulong Sid { get; set; }
        public T Ctl { get; set; }

    }

    [JsonConverter(typeof(ControlContentJsonConverter))]
    public abstract class ControlContent
    {

    }

    public class LaunchUriContent: ControlContent
    {
        public string Uri { get; set; }
        public bool Ask { get; set; }
    }

    public class ControlContentJsonConverter : JsonConverter<ControlContent>
    {
        public override ControlContent? Read(ref Utf8JsonReader reader, Type typeToConvert, JsonSerializerOptions options)
        {
            if(reader.TokenType == JsonTokenType.StartObject)
                reader.Read();
            if (reader.TokenType == JsonTokenType.PropertyName)
            {
                var name = reader.GetString();
                reader.Read();
                 var content = name switch
                {
                    "LaunchUri" => JsonSerializer.Deserialize<LaunchUriContent>(ref reader, options),
                    _ => throw new InvalidOperationException()
                };
                reader.Read();
                return content;
            }
            throw new JsonException();
        }

        public override void Write(Utf8JsonWriter writer, ControlContent value, JsonSerializerOptions options)
        {
            throw new NotImplementedException();
        }
    }
}
