using System.Diagnostics;
using FlyDrop.App.Activation;
using FlyDrop.App.Contracts.Services;
using FlyDrop.App.Core.Contracts.Services;
using FlyDrop.App.Core.Services;
using FlyDrop.App.Helpers;
using FlyDrop.App.Models;
using FlyDrop.App.Notifications;
using FlyDrop.App.Services;
using FlyDrop.App.ViewModels;
using FlyDrop.App.Views;
using FlyDrop.Core;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Hosting;
using Microsoft.UI.Xaml;

namespace FlyDrop.App;

// To learn more about WinUI 3, see https://docs.microsoft.com/windows/apps/winui/winui3/.
public partial class App : Application
{
    // The .NET Generic Host provides dependency injection, configuration, logging, and other services.
    // https://docs.microsoft.com/dotnet/core/extensions/generic-host
    // https://docs.microsoft.com/dotnet/core/extensions/dependency-injection
    // https://docs.microsoft.com/dotnet/core/extensions/configuration
    // https://docs.microsoft.com/dotnet/core/extensions/logging
    public IHost Host { get; }
   

    public static T GetRequiredService<T>()
        where T : class
    {
        return ((App)Current).Host.Services.GetRequiredService<T>();
    }

    public static WindowEx MainWindow { get; } = new MainWindow();

    public static UIElement? AppTitlebar { get; set; }

    public App()
    {
        InitializeComponent();

        // Build host
        Host = Microsoft.Extensions.Hosting.Host.
        CreateDefaultBuilder().
        UseContentRoot(AppContext.BaseDirectory).
        ConfigureServices((context, services) =>
        {
            // Default Activation Handler
            services.AddTransient<ActivationHandler<LaunchActivatedEventArgs>, DefaultActivationHandler>();

            // Other Activation Handlers
            services.AddTransient<IActivationHandler, AppNotificationActivationHandler>();

            // Services
            services.AddSingleton<IAppNotificationService, AppNotificationService>();
            services.AddSingleton<ILocalSettingsService, LocalSettingsService>();
            services.AddSingleton<IThemeSelectorService, ThemeSelectorService>();
            services.AddTransient<INavigationViewService, NavigationViewService>();

            services.AddSingleton<IActivationService, ActivationService>();
            services.AddSingleton<IPageService, PageService>();
            services.AddSingleton<INavigationService, NavigationService>();
            services.AddSingleton<Api>();
            services.AddSingleton<ApiService>();

            // Core Services
            services.AddSingleton<ISampleDataService, SampleDataService>();
            services.AddSingleton<IFileService, FileService>();

            // Hosted Services
            services.AddHostedService<ApiLifetimeManager>();


            // Views and ViewModels
            services.AddTransient<SettingsViewModel>();
            services.AddTransient<SettingsPage>();
            services.AddTransient<ShareViewModel>();
            services.AddTransient<SharePage>();
            services.AddTransient<DevicesDetailViewModel>();
            services.AddTransient<DevicesDetailPage>();
            services.AddTransient<DevicesViewModel>();
            services.AddTransient<DevicesPage>();
            services.AddTransient<ShellPage>();
            services.AddTransient<ShellViewModel>();

            // Configuration
            services.Configure<LocalSettingsOptions>(context.Configuration.GetSection(nameof(LocalSettingsOptions)));
        }).
        Build();

        // Start host
        Host.StartAsync().Wait();

        GetRequiredService<IAppNotificationService>().Initialize();

        // Register Callbacks
        MainWindow.Closed += MainWindow_Closed;
        UnhandledException += App_UnhandledException;
        AppDomain.CurrentDomain.UnhandledException += CurrentDomain_UnhandledException;
    }

    private void CurrentDomain_UnhandledException(object sender, System.UnhandledExceptionEventArgs e)
    {
        Debug.WriteLine($"Unhandled exception: {e.ExceptionObject}");

    }

    private void MainWindow_Closed(object sender, WindowEventArgs args)
    {
        if (Host != null)
        {
            Host.StopAsync().Wait();
            Host.Dispose();
        }
    }

    private void App_UnhandledException(object sender, Microsoft.UI.Xaml.UnhandledExceptionEventArgs e)
    {
        Debug.WriteLine($"Unhandled UI exception: {e.Exception}");
        // TODO: Log and handle exceptions as appropriate.
        // https://docs.microsoft.com/windows/windows-app-sdk/api/winrt/microsoft.ui.xaml.application.unhandledexception.
    }


    protected async override void OnLaunched(LaunchActivatedEventArgs args)
    {
        base.OnLaunched(args);

        // GetRequiredService<IAppNotificationService>().Show(string.Format("AppNotificationSamplePayload".GetLocalized(), AppContext.BaseDirectory));

        await GetRequiredService<IActivationService>().ActivateAsync(args);
    }

}
