using System.Collections.Generic;
using System.IO;

namespace Nasib.IconBuilder;

/// <summary>
/// One square image in an .ico file, stored as PNG.
/// </summary>
internal sealed record IconImage(int Size, byte[] PngData);

/// <summary>
/// Writes the .ico format: a six-byte header, one sixteen-byte entry per image, then the images themselves.
/// </summary>
internal static class IconFileWriter
{
    private const short ReservedValue = 0;
    private const short IconResourceType = 1;
    private const short ColorPlanes = 1;
    private const short BitsPerPixel = 32;
    private const byte NoPalette = 0;
    private const byte ReservedByte = 0;
    private const int HeaderSize = 6;
    private const int EntrySize = 16;

    /// <summary>
    /// A width or height of 256 does not fit in a byte, so the format stores it as zero.
    /// </summary>
    private const int LargestByteSize = 255;
    private const byte LargeSizeMarker = 0;

    public static void Write(string path, IReadOnlyList<IconImage> images)
    {
        Directory.CreateDirectory(Path.GetDirectoryName(Path.GetFullPath(path))!);
        using var stream = File.Create(path);
        using var writer = new BinaryWriter(stream);

        writer.Write(ReservedValue);
        writer.Write(IconResourceType);
        writer.Write((short)images.Count);

        var offset = HeaderSize + EntrySize * images.Count;
        foreach (var image in images)
        {
            var sizeByte = image.Size > LargestByteSize ? LargeSizeMarker : (byte)image.Size;
            writer.Write(sizeByte);
            writer.Write(sizeByte);
            writer.Write(NoPalette);
            writer.Write(ReservedByte);
            writer.Write(ColorPlanes);
            writer.Write(BitsPerPixel);
            writer.Write(image.PngData.Length);
            writer.Write(offset);
            offset += image.PngData.Length;
        }
        foreach (var image in images)
        {
            writer.Write(image.PngData);
        }
    }
}
