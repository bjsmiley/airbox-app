using CommunityToolkit.WinUI.UI.Animations;

using FlyDrop.App.Contracts.Services;
using FlyDrop.App.ViewModels;

using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Navigation;

namespace FlyDrop.App.Views;

public sealed partial class DevicesDetailPage : Page
{
    public DevicesDetailViewModel ViewModel
    {
        get;
    }

    public DevicesDetailPage()
    {
        ViewModel = App.GetRequiredService<DevicesDetailViewModel>();
        InitializeComponent();
    }

    protected override void OnNavigatedTo(NavigationEventArgs e)
    {
        base.OnNavigatedTo(e);
        this.RegisterElementForConnectedAnimation("animationKeyContentGrid", itemHero);
    }

    protected override void OnNavigatingFrom(NavigatingCancelEventArgs e)
    {
        base.OnNavigatingFrom(e);
        if (e.NavigationMode == NavigationMode.Back)
        {
            var navigationService = App.GetRequiredService<INavigationService>();

            if (ViewModel.Item != null)
            {
                navigationService.SetListDataItemForNextConnectedAnimation(ViewModel.Item);
            }
        }
    }
}
