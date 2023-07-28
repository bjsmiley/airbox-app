using CommunityToolkit.Mvvm.ComponentModel;

using FlyDrop.App.Contracts.ViewModels;
using FlyDrop.App.Core.Contracts.Services;
using FlyDrop.App.Core.Models;

namespace FlyDrop.App.ViewModels;

public partial class DevicesDetailViewModel : ObservableRecipient, INavigationAware
{
    private readonly ISampleDataService _sampleDataService;

    [ObservableProperty]
    private SampleOrder? item;

    public DevicesDetailViewModel(ISampleDataService sampleDataService)
    {
        _sampleDataService = sampleDataService;
    }

    public async void OnNavigatedTo(object parameter)
    {
        if (parameter is long orderID)
        {
            var data = await _sampleDataService.GetContentGridDataAsync();
            Item = data.First(i => i.OrderID == orderID);
        }
    }

    public void OnNavigatedFrom()
    {
    }
}
