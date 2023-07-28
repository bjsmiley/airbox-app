using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using FlyDrop.Core;
using FlyDrop.Core.Models.Commands;
using FlyDrop.Core.Models.Common;
using FlyDrop.Core.Models.Events;
using FlyDrop.Core.Models.Queries;
using Microsoft.Extensions.Logging;

namespace FlyDrop.App.Services;
internal class ApiService
{
    private readonly Api api;
    private readonly ILogger logger;

    public ApiService(Api api, ILoggerFactory factory)
    {
        this.api = api;
        logger = factory.CreateLogger("FlyDrop.Api");
    }

    public Task<string> StartAsync(string folder, Func<ApiEvent, Task> eventCallback)
    {
        return api.InitializeAsync(folder, eventCallback);
    }

    public async ValueTask<Configuration?> GetConfigurationAsync()
    {
        var response = await QueryAsync<GetConfigurationRequest, GetConfigurationResponse>(new GetConfigurationRequest());
        return response?.Configuration;
    }

    private async ValueTask<TRes?> QueryAsync<TReq, TRes>(TReq request) where TReq : QueryRequest where TRes : QueryResponse
    {
        if (api.IsInitalized)
        {
            var response = await api.QueryAsync<TReq, TRes>(request);
            if (response.Error is not null)
                logger.LogError("Failed to recieve query response: {0}", response.Error);
            return response.Body;
        }
        logger.LogWarning("Cannot send query. Api is uninitialized.");
        return default;

    }

    private async ValueTask<TRes?> CommandAsync<TReq, TRes>(TReq request) where TReq : CommandRequest where TRes : CommandResponse
    {
        if (api.IsInitalized)
        {
            var response = await api.CommandAsync<TReq, TRes>(request);
            if (response.Error is not null)
                logger.LogError("Failed to recieve command response: {0}", response.Error);
            return response.Body;
        }
        logger.LogWarning("Cannot send Command. Api is uninitialized.");
        return default;
    }
}
