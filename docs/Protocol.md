# Protocol

## Common Header
Every Frame will contain the Common Header followed by the payload.

Name | Length (bytes) | Description
---  | ---            | ---
Signature | 2 | Fixed signature, which is always 0x4040.
MessageLength | 2 | Entire message length in bytes including signature.
MessageType | 1 | Indicates current message type.


<!-- RequestId | 8 | A monotonically increasing number generated on the sending side, that uniquely identifies the message. It can then be used to correlate response messages to their corresponding request message. -->

## Discovery

### Discovery Messages
A device sends out a presence request and a second device responds with a presence response.

#### Presence Request
This is the message any device can subscribe to and respond to in order to participate in the Discovery Protocol.

Name | Length (bytes) | Description
---  | ---            | ---
DiscoveryType | 1 | Indicates type of discovery message (0).

#### Presence Response
When a device receives a presence request, it responds with a presence response to notify that it's available.

Name | Length (bytes) | Description
---  | ---            | ---
DiscoveryType | 1 | Indicates type of discovery message (1). |
DeviceType | 2 | SKU of the device. |
DeviceNameLength | 2 | Length of the machine name of the device. |
DeviceName | variable | The character representation of the name of the device. |
DeviceId | 40 | The peer id of this device. |
DeviceAddressLength | 2 | the length of the valid device address IP and port string. |
DeviceAddress | variable | the device address. |

### Connection Messages
These are the messages during authentication of a connection when a device is discovered.

#### Connection Request
Client initiates a connection request with a host device. 

Name | Length (bytes) | Description
---  | ---            | ---
ConnectMessageType | 1 | Indicates the current connection message type (0) |
| PeerId | 40 | The client's peer id |
| HMAC | 32 | HMAC of the client's peer id using the current totp passcode as the key | 

### Connection Response
The host responds with a connection response message after validating the connection request Auth Code.

Name | Length (bytes) | Description
---  | ---            | ---
ConnectMessageType | 1 | Indicates the current connection message type (1) |
| HMAC | 32 | HMAC of the host's peer id using the current totp passcode as the key |

### Connection Complete Request
The client informs the host connecting has been successful.

Name | Length (bytes) | Description
---  | ---            | ---
ConnectMessageType | 1 | Indicates the current connection message type (2) |

### Connection Complete Response
The client informs the host connecting has been successful.

Name | Length (bytes) | Description
---  | ---            | ---
ConnectMessageType | 1 | Indicates the current connection message type (3) |

### Connection Failure
The host or the client responds with a connection failure if something when wrong during connecting phase.

Name | Length (bytes) | Description
---  | ---            | ---
ConnectMessageType | 1 | Indicates the current connection message type (4) |
| Result | 4 | An implementation-specific field containing the result. A value of zero indicates success. |
