namespace FlyDrop.Core
{
    partial struct Buffer
    {
        public unsafe Span<byte> AsSpan()
        {
            return new Span<byte>(ptr, len);
        }

        public unsafe Span<T> AsSpan<T>()
        {
            return MemoryMarshal.CreateSpan(ref Unsafe.AsRef<T>(ptr), len / Unsafe.SizeOf<T>());
        }
    }
}