using System;
using System.Collections.Generic;
using System.Linq;
using System.Runtime.InteropServices;
using System.Text;
using System.Threading.Tasks;

namespace FlyDrop.Core
{
    public delegate void CallbackCnt(string content);
    public delegate void Callback();

    internal static unsafe class Native
    {
        const string __DllName = "ffi";

        [DllImport(__DllName, EntryPoint = "init", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void Initialize(string data_dir, CallbackCnt on_event, Callback on_ready);

        /*        public static extern void Initialize(string data_dir, delegate* unmanaged[Cdecl]<string> on_event, delegate* unmanaged[Cdecl]<void> on_ready);
        */
        [DllImport(__DllName, EntryPoint = "query", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void Query(string msg, CallbackCnt callback);

        [DllImport(__DllName, EntryPoint = "cmd", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        public static extern void Cmd(string msg, CallbackCnt callback);

    }
}
