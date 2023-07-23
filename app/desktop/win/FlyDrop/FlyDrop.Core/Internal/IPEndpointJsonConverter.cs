using System;
using System.Collections.Generic;
using System.Globalization;
using System.Linq;
using System.Net.Sockets;
using System.Net;
using System.Text;
using System.Text.Json.Serialization;
using System.Text.Json;
using System.Threading.Tasks;
using System.Diagnostics;
using System.Reflection;
using System.Buffers;

namespace FlyDrop.Core.Internal
{
    public class IPEndPointJsonConverter : JsonConverter<IPEndPoint>
    {

        /// <inheritdoc/>
        public override IPEndPoint Read(ref Utf8JsonReader reader, Type typeToConvert, JsonSerializerOptions options)
        {
            if (reader.TokenType != JsonTokenType.String)
                throw ThrowHelper.GenerateJsonException_DeserializeUnableToConvertValue(typeof(IPEndPoint));
            Span<char> charData = stackalloc char[53];
            int count = Encoding.UTF8.GetChars(
                reader.HasValueSequence ? reader.ValueSequence.ToArray() : reader.ValueSpan,
                charData);

            int addressLength = count;
            int lastColonPos = charData.LastIndexOf(':');

            if (lastColonPos > 0)
            {
                if (charData[lastColonPos - 1] == ']')
                {
                    addressLength = lastColonPos;
                }
                else if (charData[..lastColonPos].LastIndexOf(':') == -1)
                {
                    // Look to see if this is IPv4 with a port (IPv6 will have another colon)
                    addressLength = lastColonPos;
                }
            }

            if (!IPAddress.TryParse(charData[..addressLength], out IPAddress? address))
                throw ThrowHelper.GenerateJsonException_DeserializeUnableToConvertValue(typeof(IPEndPoint));

            uint port = 0;
            return addressLength == charData.Length ||
                (uint.TryParse(charData[(addressLength + 1)..], NumberStyles.None, CultureInfo.InvariantCulture, out port) && port <= IPEndPoint.MaxPort)
                ? new IPEndPoint(address, (int)port)
                : throw ThrowHelper.GenerateJsonException_DeserializeUnableToConvertValue(typeof(IPEndPoint));
        }

        /// <inheritdoc/>
        public override void Write(Utf8JsonWriter writer, IPEndPoint value, JsonSerializerOptions options)
#pragma warning disable CA1062 // Don't perform checks for performance. Trust our callers will be nice.
        {
            bool isIpv6 = value.AddressFamily == AddressFamily.InterNetworkV6;
            Span<char> data = isIpv6
                ? stackalloc char[21]
                : stackalloc char[53];
            int offset = 0;
            if (isIpv6)
            {
                data[0] = '[';
                offset++;
            }
            if (!value.Address.TryFormat(data[offset..], out int addressCharsWritten))
                throw new JsonException($"IPEndPoint [{value}] could not be written to JSON.");
            offset += addressCharsWritten;
            if (isIpv6)
            {
                data[offset++] = ']';
            }
            data[offset++] = ':';
            if (!value.Port.TryFormat(data[offset..], out int portCharsWritten))
                throw new JsonException($"IPEndPoint [{value}] could not be written to JSON.");
            writer.WriteStringValue(data[..(offset + portCharsWritten)]);
        }
#pragma warning restore CA1062 // Validate arguments of public methods
    }

    internal static class ThrowHelper
    {
        private static readonly PropertyInfo? s_JsonException_AppendPathInformation
            = typeof(JsonException).GetProperty("AppendPathInformation", BindingFlags.NonPublic | BindingFlags.Instance);

        /// <summary>
        /// Generate a <see cref="JsonException"/> using the internal
        /// <c>JsonException.AppendPathInformation</c> property that will
        /// eventually include the JSON path, line number, and byte position in
        /// line.
        /// <para>
        /// The final message of the exception looks like: The JSON value could
        /// not be converted to {0}. Path: $.{JSONPath} | LineNumber:
        /// {LineNumber} | BytePositionInLine: {BytePositionInLine}.
        /// </para>
        /// </summary>
        /// <param name="propertyType">Property type.</param>
        /// <returns><see cref="JsonException"/>.</returns>
        public static JsonException GenerateJsonException_DeserializeUnableToConvertValue(Type propertyType)
        {
            Debug.Assert(s_JsonException_AppendPathInformation != null);

            JsonException jsonException = new($"The JSON value could not be converted to {propertyType}.");
            s_JsonException_AppendPathInformation?.SetValue(jsonException, true);
            return jsonException;
        }

        /// <summary>
        /// Generate a <see cref="JsonException"/> using the internal
        /// <c>JsonException.AppendPathInformation</c> property that will
        /// eventually include the JSON path, line number, and byte position in
        /// line.
        /// <para>
        /// The final message of the exception looks like: The JSON value '{1}'
        /// could not be converted to {0}. Path: $.{JSONPath} | LineNumber:
        /// {LineNumber} | BytePositionInLine: {BytePositionInLine}.
        /// </para>
        /// </summary>
        /// <param name="propertyType">Property type.</param>
        /// <param name="propertyValue">Value that could not be parsed into
        /// property type.</param>
        /// <param name="innerException">Optional inner <see cref="Exception"/>.</param>
        /// <returns><see cref="JsonException"/>.</returns>
        public static JsonException GenerateJsonException_DeserializeUnableToConvertValue(
            Type propertyType,
            string propertyValue,
            Exception? innerException = null)
        {
            Debug.Assert(s_JsonException_AppendPathInformation != null);

            JsonException jsonException = new(
                $"The JSON value '{propertyValue}' could not be converted to {propertyType}.",
                innerException);
            s_JsonException_AppendPathInformation?.SetValue(jsonException, true);
            return jsonException;
        }
    }
}
