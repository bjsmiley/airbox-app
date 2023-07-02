using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace FlyDrop.Core
{
    public class Api : IDisposable
    {
        private readonly Mutex _mutex;
        public Api()
        {
            var id = Process.GetCurrentProcess().Id;
            _mutex = new Mutex(true, $"flydrop-{id}");
        }

        public Task CommandAsync()
        {

        }

        public void Dispose()
        {
            _mutex?.Dispose();
        }
    }
}
