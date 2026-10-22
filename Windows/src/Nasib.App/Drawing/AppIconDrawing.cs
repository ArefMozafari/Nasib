using System.Windows;
using System.Windows.Media;
using System.Windows.Media.Imaging;

namespace Nasib.App.Drawing;

/// <summary>
/// The app icon: three white waves on a flat blue rounded square, drawn from the same paths as
/// Design/AppIcon.icon/Assets/waves.svg. The app draws it at run time and Nasib.IconBuilder writes the .exe icon from it.
/// </summary>
public static class AppIconDrawing
{
    /// <summary>
    /// The paths are laid out on this square canvas, as in waves.svg.
    /// </summary>
    private const double CanvasSize = 1024;
    private const double CornerRadius = 192;
    private const double WaveStrokeWidth = 72;

    /// <summary>
    /// At 96 DPI one device-independent unit is one pixel, so the bitmap is exactly the size asked for.
    /// </summary>
    private const double DeviceIndependentDpi = 96;

    // Each wave spans x 216..808 (center 512); the three sit at y 372/512/652 (center 512).
    private static readonly string[] WavePaths =
    [
        "M216 372 C 315 310, 413 310, 512 372 S 709 434, 808 372",
        "M216 512 C 315 450, 413 450, 512 512 S 709 574, 808 512",
        "M216 652 C 315 590, 413 590, 512 652 S 709 714, 808 652",
    ];

    /// <summary>
    /// The middle of the macOS icon's blue gradient, since a flat icon has one fill.
    /// </summary>
    private static readonly Color BackgroundColor = Color.FromRgb(0x1F, 0x5F, 0xBE);

    private static readonly Color WaveColor = Colors.White;

    /// <summary>
    /// A resolution-independent image for the window header, the tray menu and the About window.
    /// </summary>
    public static ImageSource ImageSource { get; } = CreateImageSource();

    public static DrawingGroup CreateDrawing()
    {
        var background = new SolidColorBrush(BackgroundColor);
        var pen = new Pen(new SolidColorBrush(WaveColor), WaveStrokeWidth)
        {
            StartLineCap = PenLineCap.Round,
            EndLineCap = PenLineCap.Round,
            LineJoin = PenLineJoin.Round,
        };
        var drawing = new DrawingGroup();
        drawing.Children.Add(new GeometryDrawing(
            background,
            null,
            new RectangleGeometry(new Rect(0, 0, CanvasSize, CanvasSize), CornerRadius, CornerRadius)));
        foreach (var path in WavePaths)
        {
            drawing.Children.Add(new GeometryDrawing(null, pen, Geometry.Parse(path)));
        }
        drawing.Freeze();
        return drawing;
    }

    /// <summary>
    /// Renders the icon to a square bitmap of the given size in pixels.
    /// </summary>
    public static BitmapSource Render(int pixelSize)
    {
        var visual = new DrawingVisual();
        using (var context = visual.RenderOpen())
        {
            context.PushTransform(new ScaleTransform(pixelSize / CanvasSize, pixelSize / CanvasSize));
            context.DrawDrawing(CreateDrawing());
        }
        var bitmap = new RenderTargetBitmap(pixelSize, pixelSize, DeviceIndependentDpi, DeviceIndependentDpi, PixelFormats.Pbgra32);
        bitmap.Render(visual);
        bitmap.Freeze();
        return bitmap;
    }

    private static ImageSource CreateImageSource()
    {
        var image = new DrawingImage(CreateDrawing());
        image.Freeze();
        return image;
    }
}
