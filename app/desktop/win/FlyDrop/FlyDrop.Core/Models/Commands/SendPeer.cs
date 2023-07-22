using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Text.Json.Serialization;
using System.Threading.Tasks;

namespace FlyDrop.Core.Models.Commands
{
    public class SendPeerRequest<T>: CommandRequest
    {
        [JsonPropertyName("SendPeer")]

        public SendContent<T> Content { get; set; }
    }

    public class SendContent<T>
    {
        public string Peer { get; set; }
        [JsonPropertyName("req")]
        public T Request { get; set; }
    }

    public class LaunchUri
    {
        [JsonPropertyName("LaunchUri")]
        public string Uri { get; set; }
    }


}
