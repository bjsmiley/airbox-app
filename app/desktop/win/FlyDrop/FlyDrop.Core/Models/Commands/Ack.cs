using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Text.Json.Serialization;
using System.Threading.Tasks;

namespace FlyDrop.Core.Models.Commands
{
    public class AckRequest: CommandRequest
    {
        [JsonPropertyName("Ack")]
        public AckContent Ack { get; set; }
    }

    public class AckContent
    {
        public string Peer { get; set; }
        public ulong Sid { get; set; }
        public Ack Ack { get; set; }
    }

    [JsonConverter(typeof(JsonStringEnumConverter))]
    public enum Ack
    {
        Accepted,
        Cancelled
    }
}
