using FlyDrop.App.ViewModels;

using Microsoft.UI.Xaml.Controls;

namespace FlyDrop.App.Views;

public sealed partial class SharePage : Page
{
    public ShareViewModel ViewModel
    {
        get;
    }

    public SharePage()
    {
        ViewModel = App.GetRequiredService<ShareViewModel>();
        InitializeComponent();
    }
}
