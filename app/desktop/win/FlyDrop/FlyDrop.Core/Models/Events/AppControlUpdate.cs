using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Text.Json.Serialization;
using System.Threading.Tasks;

namespace FlyDrop.Core.Models.Events
{
    public class AppControlUpdateEvent: ApiEvent
    {
        public string Peer { get; set; }
        public AppControlStatus Status { get; set; }
    }

    [JsonConverter(typeof(JsonStringEnumConverter))]
    public enum AppControlStatus
    {
        Waiting,
        Success,
        Cancelled,
        Failed,
    }


}
