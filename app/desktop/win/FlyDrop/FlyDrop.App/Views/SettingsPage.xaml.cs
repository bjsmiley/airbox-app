using FlyDrop.App.ViewModels;

using Microsoft.UI.Xaml.Controls;

namespace FlyDrop.App.Views;

// TODO: Set the URL for your privacy policy by updating SettingsPage_PrivacyTermsLink.NavigateUri in Resources.resw.
public sealed partial class SettingsPage : Page
{
    public SettingsViewModel ViewModel
    {
        get;
    }

    public SettingsPage()
    {
        ViewModel = App.GetRequiredService<SettingsViewModel>();
        InitializeComponent();
    }
}
