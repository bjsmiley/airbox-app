using FlyDrop.Core.Models.Common;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Text.Json;
using System.Text.Json.Serialization;
using System.Threading.Tasks;

namespace FlyDrop.Core.Models.Queries
{
    [JsonConverter(typeof(GetConfigurationRequestJsonConverter))]
    public class GetConfigurationRequest: QueryRequest
    {
    }

    public class GetConfigurationResponse: QueryResponse 
    {
        [JsonPropertyName("Conf")]
        public Configuration Configuration { get; set; }
    }


    internal class GetConfigurationRequestJsonConverter: JsonConverter<GetConfigurationRequest>
    {
        public override GetConfigurationRequest? Read(ref Utf8JsonReader reader, Type typeToConvert, JsonSerializerOptions options)
        {
            throw new NotImplementedException();
        }

        public override void Write(Utf8JsonWriter writer, GetConfigurationRequest value, JsonSerializerOptions options)
        {
            writer.WriteRawValue("\"GetConf\"");
        }
    }
}
