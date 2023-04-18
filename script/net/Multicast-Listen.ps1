param(
    [Parameter(Mandatory)]
    [string] $MulticastIpAddress,
    [Parameter(Mandatory)]
    [int] $Port
)

$client = [System.Net.Sockets.UdpClient]::New()
    
$client.ExclusiveAddressUse = $false;
$localEp = [System.Net.IPEndPoint]::New([IPAddress]::Any, $Port);

$client.Client.SetSocketOption([System.Net.Sockets.SocketOptionLevel]::Socket, [System.Net.Sockets.SocketOptionName]::ReuseAddress, $true);
$client.ExclusiveAddressUse = $false;

$client.Client.Bind($localEp);

$multicastaddress = [IPAddress]::Parse($MulticastIpAddress);
$client.JoinMulticastGroup($multicastaddress);

[Console]::WriteLine('Listening this will never quit so you will need to Ctrl-Break it');

while ($true) {
    [Byte[]]$data = $client.Receive([ref]$localEp);
    $strData = [Encoding.Unicode]::GetString($data);
    [Console]::WriteLine($strData);
}
