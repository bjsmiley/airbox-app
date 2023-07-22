using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Text.Json.Serialization;
using System.Text.Json;
using System.Threading.Tasks;

namespace FlyDrop.Core.Models.Queries
{
    [JsonConverter(typeof(GetSharableQrCodeRequestJsonConverter))]
    public class GetSharableQrCodeRequest: QueryRequest
    {
        // GetSharableQrCode
    }

    public class GetSharableQrCodeResponse: QueryResponse
    {
        public string? GetSharableQrCode { get; set; }
    }

    internal class GetSharableQrCodeRequestJsonConverter: JsonConverter<GetSharableQrCodeRequest>
    {
        public override GetSharableQrCodeRequest? Read(ref Utf8JsonReader reader, Type typeToConvert, JsonSerializerOptions options)
        {
            throw new NotImplementedException();
        }

        public override void Write(Utf8JsonWriter writer, GetSharableQrCodeRequest value, JsonSerializerOptions options)
        {
            writer.WriteRawValue("\"GetSharableQrCode\"");
        }
    }
}
