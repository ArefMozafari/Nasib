using System.Runtime.InteropServices;
using System.Windows.Media;
using System.Windows.Media.Imaging;
using DrawingBitmap = System.Drawing.Bitmap;
using DrawingIcon = System.Drawing.Icon;
using DrawingImageLockMode = System.Drawing.Imaging.ImageLockMode;
using DrawingPixelFormat = System.Drawing.Imaging.PixelFormat;
using DrawingRectangle = System.Drawing.Rectangle;

namespace Nasib.App.Drawing;

/// <summary>
/// Turns a WPF bitmap into the Windows Forms icon the tray needs.
/// </summary>
public static class IconConversion
{
    private const int BytesPerPixel = 4;

    /// <summary>
    /// Returns an icon that owns its own handle, so disposing it frees everything.
    /// </summary>
    public static DrawingIcon ToIcon(BitmapSource source)
    {
        // The tray expects straight alpha, while WPF renders premultiplied.
        var converted = new FormatConvertedBitmap(source, PixelFormats.Bgra32, null, 0);
        var width = converted.PixelWidth;
        var height = converted.PixelHeight;
        var stride = width * BytesPerPixel;
        var pixels = new byte[stride * height];
        converted.CopyPixels(pixels, stride, 0);

        using var bitmap = new DrawingBitmap(width, height, DrawingPixelFormat.Format32bppArgb);
        var data = bitmap.LockBits(new DrawingRectangle(0, 0, width, height), DrawingImageLockMode.WriteOnly, DrawingPixelFormat.Format32bppArgb);
        try
        {
            for (var row = 0; row < height; row++)
            {
                Marshal.Copy(pixels, row * stride, data.Scan0 + row * data.Stride, stride);
            }
        }
        finally
        {
            bitmap.UnlockBits(data);
        }

        // GetHicon hands back a handle nobody owns; the clone gets its own copy, and the original is destroyed.
        var handle = bitmap.GetHicon();
        try
        {
            using var borrowed = DrawingIcon.FromHandle(handle);
            return (DrawingIcon)borrowed.Clone();
        }
        finally
        {
            NativeMethods.DestroyIcon(handle);
        }
    }
}
