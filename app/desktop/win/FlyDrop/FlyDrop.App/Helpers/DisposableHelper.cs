using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using FlyDrop.Core;

namespace FlyDrop.App.Helpers;
internal class DisposableHelper : IDisposable
{
    private readonly Api api;

    public DisposableHelper(Api api)
    {
        this.api = api;
    }

    public void Dispose()
    {
        api.Dispose();
    }
}
