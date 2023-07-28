using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using FlyDrop.App.Contracts.Services;
using FlyDrop.Core.Models.Events;
using Microsoft.Extensions.Hosting;
using Microsoft.Extensions.Logging;

namespace FlyDrop.App.Services;
internal class ApiLifetimeManager : BackgroundService
{
    private readonly ApiService apiService;
    private readonly ILogger logger;
    private readonly ILocalSettingsService localSettingsService;

    public ApiLifetimeManager(ApiService apiService, ILoggerFactory factory, ILocalSettingsService localSettingsService)
    {
        this.apiService = apiService;
        this.logger = factory.CreateLogger("FlyDrop.Api.Lifetime");
        this.localSettingsService = localSettingsService;
    }

    private Task OnEvent(ApiEvent ev)
    {
        logger.LogInformation("Recieved event: {0}", ev);
        return Task.CompletedTask;
    }

    protected override async Task ExecuteAsync(CancellationToken stoppingToken)
    {
        var folder = localSettingsService.GetLocalApplicationDataFolder();
        var result = await apiService.StartAsync(folder, OnEvent);
        if (result != "Initialized")
        {
            logger.LogError("Core api failed to start: {0}", result);
        }
        else
        {
            logger.LogInformation("Core api started");

        }


    }
}
