using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Windows.Media.Imaging;
using Nasib.App.Drawing;

namespace Nasib.IconBuilder;

/// <summary>
/// Usage: Nasib.IconBuilder &lt;output.ico&gt;. Renders the app icon at every size Windows asks for and writes one .ico.
/// </summary>
internal static class Program
{
    /// <summary>
    /// Small sizes for lists and the taskbar at each scale factor, up to the 256-pixel tile Explorer uses.
    /// </summary>
    private static readonly int[] IconSizes = [16, 20, 24, 32, 40, 48, 64, 256];

    private const int SuccessExitCode = 0;
    private const int UsageExitCode = 1;

    [STAThread]
    private static int Main(string[] arguments)
    {
        if (arguments.Length != 1)
        {
            Console.Error.WriteLine("usage: Nasib.IconBuilder <output.ico>");
            return UsageExitCode;
        }
        var images = IconSizes.Select(size => new IconImage(size, EncodePng(AppIconDrawing.Render(size)))).ToList();
        IconFileWriter.Write(arguments[0], images);
        Console.WriteLine($"wrote {arguments[0]} ({string.Join(", ", IconSizes)} px)");
        return SuccessExitCode;
    }

    private static byte[] EncodePng(BitmapSource bitmap)
    {
        var encoder = new PngBitmapEncoder();
        encoder.Frames.Add(BitmapFrame.Create(bitmap));
        using var stream = new MemoryStream();
        encoder.Save(stream);
        return stream.ToArray();
    }
}
