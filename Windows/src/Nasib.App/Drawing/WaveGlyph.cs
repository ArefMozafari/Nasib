using System.Windows;
using System.Windows.Media;
using System.Windows.Media.Imaging;

namespace Nasib.App.Drawing;

public enum GlyphState
{
    Connected,
    Searching,
    Down,
}

/// <summary>
/// The tray glyph, drawn from the paths in Design/MenuBarConnected.svg, MenuBarSearching.svg and MenuBarDown.svg:
/// solid waves when connected, dashed while searching, crossed out when down.
/// </summary>
public static class WaveGlyph
{
    private const double StrokeWidth = 3.2;
    private const double DashLength = 4;
    private const double DashGap = 3;

    /// <summary>
    /// The SVGs use a 36-unit square with room around the waves for the macOS menu bar. The tray draws the
    /// 28-unit square from 4 to 32, which still holds the slash and its round caps, so the glyph fills more of
    /// the 16-pixel tray icon.
    /// </summary>
    private const double ViewBoxOrigin = 4;
    private const double ViewBoxSize = 28;

    private const double DeviceIndependentDpi = 96;

    // Each wave spans x 6..30 (center 18); the three sit at y 11/18/25 (center 18).
    private static readonly string[] WavePaths =
    [
        "M6 11 C 10 8.5, 14 8.5, 18 11 S 26 13.5, 30 11",
        "M6 18 C 10 15.5, 14 15.5, 18 18 S 26 20.5, 30 18",
        "M6 25 C 10 22.5, 14 22.5, 18 25 S 26 27.5, 30 25",
    ];

    private const string SlashPath = "M8 30 L 28 6";

    public static DrawingGroup CreateDrawing(GlyphState state, Color color)
    {
        var brush = new SolidColorBrush(color);
        var wavePen = CreatePen(brush);
        if (state == GlyphState.Searching)
        {
            // WPF measures dashes in stroke widths, while the SVG measures them in user units.
            wavePen.DashStyle = new DashStyle([DashLength / StrokeWidth, DashGap / StrokeWidth], 0);
            wavePen.DashCap = PenLineCap.Round;
        }
        var drawing = new DrawingGroup();
        foreach (var path in WavePaths)
        {
            drawing.Children.Add(new GeometryDrawing(null, wavePen, Geometry.Parse(path)));
        }
        if (state == GlyphState.Down)
        {
            drawing.Children.Add(new GeometryDrawing(null, CreatePen(brush), Geometry.Parse(SlashPath)));
        }
        drawing.Freeze();
        return drawing;
    }

    public static BitmapSource Render(GlyphState state, Color color, int pixelSize)
    {
        var scale = pixelSize / ViewBoxSize;
        var visual = new DrawingVisual();
        using (var context = visual.RenderOpen())
        {
            context.PushTransform(new ScaleTransform(scale, scale));
            context.PushTransform(new TranslateTransform(-ViewBoxOrigin, -ViewBoxOrigin));
            context.PushClip(new RectangleGeometry(new Rect(ViewBoxOrigin, ViewBoxOrigin, ViewBoxSize, ViewBoxSize)));
            context.DrawDrawing(CreateDrawing(state, color));
        }
        var bitmap = new RenderTargetBitmap(pixelSize, pixelSize, DeviceIndependentDpi, DeviceIndependentDpi, PixelFormats.Pbgra32);
        bitmap.Render(visual);
        bitmap.Freeze();
        return bitmap;
    }

    private static Pen CreatePen(Brush brush)
    {
        return new Pen(brush, StrokeWidth)
        {
            StartLineCap = PenLineCap.Round,
            EndLineCap = PenLineCap.Round,
            LineJoin = PenLineJoin.Round,
        };
    }
}
