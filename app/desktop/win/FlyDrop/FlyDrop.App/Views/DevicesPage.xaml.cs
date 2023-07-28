using FlyDrop.App.ViewModels;

using Microsoft.UI.Xaml.Controls;

namespace FlyDrop.App.Views;

public sealed partial class DevicesPage : Page
{
    public DevicesViewModel ViewModel
    {
        get;
    }

    public DevicesPage()
    {
        ViewModel = App.GetRequiredService<DevicesViewModel>();
        InitializeComponent();
    }
}
