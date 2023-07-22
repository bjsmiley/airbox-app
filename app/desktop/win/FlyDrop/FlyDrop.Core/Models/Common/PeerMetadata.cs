using System;
using System.Collections.Generic;
using System.Linq;
using System.Net;
using System.Text;
using System.Text.Json.Serialization;
using System.Threading.Tasks;

namespace FlyDrop.Core.Models.Common
{
    public enum DeviceType : ushort
    {
        Unknown = 0,
        // XboxOne = 1,
        AppleiPhone = 6,
        AppleiPad = 7,
        AndroidDevice = 8,
        Windows10Desktop = 9,
        // Windows10Phone = 11,
        LinuxDevice = 12,
        // WindowsIoT = 13,
        // SurfaceHub = 14,
        WindowsLaptop = 15,
        // WindowsTablet = 16
    }

    public class PeerMetadata
    {
        public string Name { get; set; }
        public string Id { get; set; }
        [JsonPropertyName("typ")]
        public DeviceType Type { get; set; }
        [JsonPropertyName("addr")]
        public IPEndPoint Address { get; set; }
    }
}
