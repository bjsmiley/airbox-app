// See https://aka.ms/new-console-template for more information
using FlyDrop.Core;
using FlyDrop.Core.Models.Commands;
using FlyDrop.Core.Models.Queries;

Console.WriteLine("Hello, World!");
/*Environment.SetEnvironmentVariable("RUST_BACKTRACE", "1");
*/


var api = await Api.CreateAsync(Path.GetTempPath(), (x) => Console.WriteLine(x));
// var res = await api.QueryAsync<GetConfigurationRequest, GetConfigurationResponse>(new GetConfigurationRequest());

/*
var res = await api.CommandAsync<SendPeerRequest<LaunchUri>, CommandResponse>(new SendPeerRequest<LaunchUri>
{
    Content = new Content<LaunchUri>
    {
        Peer = "0000000000000000000000000000000000000000",
        Request = new LaunchUri
        {
            Uri = "https://google.com"
        }
    }
});
*/
var res = await api.CommandAsync<AckRequest, CommandResponse>(new AckRequest
{
    Ack = new AckContent
    {
        Peer = "0000000000000000000000000000000000000000",
        Sid = 0,
        Ack = Ack.Accepted
    }
});

Console.WriteLine(res.Error);



// from core
/*
{"SendPeer":{"peer":"0000000000000000000000000000000000000000","req":{"LaunchUri":"https://google.com"}}}
 */