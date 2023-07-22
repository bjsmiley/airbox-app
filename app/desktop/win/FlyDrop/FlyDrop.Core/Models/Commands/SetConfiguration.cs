using FlyDrop.Core.Models.Common;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Text.Json.Serialization;
using System.Threading.Tasks;

namespace FlyDrop.Core.Models.Commands
{
    internal class SetConfigurationRequest : CommandRequest
    {
        [JsonPropertyName("SetConf")]
        public Configuration Configuration { get; set; }
    }
}
