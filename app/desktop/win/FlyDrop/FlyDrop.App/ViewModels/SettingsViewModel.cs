using System.Reflection;
using System.Windows.Input;

using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using CommunityToolkit.WinUI;
using FlyDrop.App.Contracts.Services;
using FlyDrop.App.Helpers;
using FlyDrop.Core;
using FlyDrop.Core.Models.Common;
using FlyDrop.Core.Models.Queries;
using Microsoft.UI.Dispatching;
using Microsoft.UI.Xaml;

using Windows.ApplicationModel;

namespace FlyDrop.App.ViewModels;

public partial class SettingsViewModel : ObservableRecipient
{
    private readonly IThemeSelectorService _themeSelectorService;
    private readonly Api api;


    [ObservableProperty]
    private ElementTheme _elementTheme;

    [ObservableProperty]
    private string _versionDescription;

    [ObservableProperty]
    private string _name;

    [ObservableProperty]
    private string _peerId;

    [ObservableProperty]
    private bool _autoAccept;

    public ICommand SwitchThemeCommand
    {
        get;
    }

    public SettingsViewModel(IThemeSelectorService themeSelectorService, Api api)
    {
        _themeSelectorService = themeSelectorService;
        this.api = api;
        _elementTheme = _themeSelectorService.Theme;
        _versionDescription = GetVersionDescription();

        SwitchThemeCommand = new RelayCommand<ElementTheme>(
            async (param) =>
            {
                if (ElementTheme != param)
                {
                    ElementTheme = param;
                    await _themeSelectorService.SetThemeAsync(param);
                }
            });

        var queue = DispatcherQueue.GetForCurrentThread();
        _ = Task.Run(async () =>
        {
            var response = await api.QueryAsync<GetConfigurationRequest, GetConfigurationResponse>(new GetConfigurationRequest());
            if (response.Body != null)
            {
                await queue.EnqueueAsync(() => { 
                    var config = response.Body.Configuration;
                    _peerId = config.Id;
                    _name = config.Name;
                    _autoAccept = config.AutoAccept;
                });
            }
        });
    }

    private static string GetVersionDescription()
    {
        Version version;

        if (RuntimeHelper.IsMSIX)
        {
            var packageVersion = Package.Current.Id.Version;

            version = new(packageVersion.Major, packageVersion.Minor, packageVersion.Build, packageVersion.Revision);
        }
        else
        {
            version = Assembly.GetExecutingAssembly().GetName().Version!;
        }

        return $"{ResourceExtensions.GetLocalized("AppDisplayName")} - {version.Major}.{version.Minor}.{version.Build}.{version.Revision}";
    }
}
