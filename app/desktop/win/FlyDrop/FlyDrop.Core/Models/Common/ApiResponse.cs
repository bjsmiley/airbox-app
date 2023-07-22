using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Text.Json.Serialization;
using System.Threading.Tasks;

namespace FlyDrop.Core.Models.Common
{
    public class ApiResponse<T>
    {
        [JsonPropertyName("err")]
        public string? Error { get; set; }
        [JsonPropertyName("res")]
        public T? Body { get; set; }
    }
}
