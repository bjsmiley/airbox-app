using FlyDrop.Core.Internal;
using FlyDrop.Core.Models.Commands;
using FlyDrop.Core.Models.Common;
using FlyDrop.Core.Models.Queries;
using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using System.Text;
using System.Text.Json;
using System.Text.Json.Serialization;
using System.Threading.Channels;
using System.Threading.Tasks;

namespace FlyDrop.Core
{

    public class Api : IDisposable
    {
        private readonly Mutex _mutex;
        private readonly JsonSerializerOptions _options;
        Api(JsonSerializerOptions? options = null)
        {
            var id = Process.GetCurrentProcess().Id;
            _mutex = new Mutex(true, $"flydrop-{id}");
            _options = options ?? new JsonSerializerOptions(JsonSerializerOptions.Default);
            _options.PropertyNamingPolicy = new JsonSnakeCaseLowerNamingPolicy();
        }

        public static async Task<Api> CreateAsync(string directory, CallbackCnt eventCallback, JsonSerializerOptions? options = null)
        {
            var api = new Api(options);
            var channel = Channel.CreateBounded<object>(1);

            Native.Initialize(directory,eventCallback, () =>
            {
                channel.Writer.WriteAsync(new object());
            });

            await channel.Reader.ReadAsync();
            return api;

        }
        /*
           public ValueTask<string> CommandAsync(string json)
                {
                    var channel = Channel.CreateBounded<string>(1);
                    Native.Cmd(json, (res) =>
                    {
                        channel.Writer.WriteAsync(res);
                    });
                    return channel.Reader.ReadAsync();
                }*/
        public ValueTask<ApiResponse<TRes>> QueryAsync<TReq, TRes>(TReq request) where TReq : QueryRequest where TRes : QueryResponse
        {
            var channel = Channel.CreateBounded<ApiResponse<TRes>>(1);
            var json = JsonSerializer.Serialize(request, _options);

            Native.Query(json, (res) =>
            {
                try
                {
                    Console.WriteLine(res);
                    var response = JsonSerializer.Deserialize<ApiResponse<TRes>>(res, _options);
                    channel.Writer.WriteAsync(response);
                }
                catch (Exception ex)
                {
                    Console.Error.WriteLine("Failed to read query response from core: {}", res);
                    channel.Writer.Complete(ex);

                }
            });
            return channel.Reader.ReadAsync();
        }

        public ValueTask<ApiResponse<TRes>> CommandAsync<TReq, TRes>(TReq request) where TReq : CommandRequest where TRes : CommandResponse
        {
            var channel = Channel.CreateBounded<ApiResponse<TRes>>(1);
            var json = JsonSerializer.Serialize(request, _options);

            Native.Cmd(json, (res) =>
            {
                try
                {
                    Console.WriteLine(res);
                    var response = JsonSerializer.Deserialize<ApiResponse<TRes>>(res, _options);
                    channel.Writer.WriteAsync(response);
                }
                catch (Exception ex)
                {
                    Console.Error.WriteLine("Failed to read cmd response from core: {}", res);
                    channel.Writer.Complete(ex);

                }
            });
            return channel.Reader.ReadAsync();
        }

        public void Dispose()
        {
            _mutex?.Dispose();
        }
    }
}
