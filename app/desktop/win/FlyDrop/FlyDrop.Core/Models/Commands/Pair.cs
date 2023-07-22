using FlyDrop.Core.Models.Common;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace FlyDrop.Core.Models.Commands
{
    public class PairRequest: CommandRequest
    {
        public QrPayload Pair { get; set; }
    }

    public class QrPayload
    {
        public string Secret { get; set; }
        public PeerMetadata Peer { get; set; }
    }
}
