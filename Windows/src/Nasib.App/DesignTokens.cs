using System.Windows;
using System.Windows.Media;

namespace Nasib.App;

/// <summary>
/// Sizes and spacing for the window, the tray menu and the About window, in one place.
/// </summary>
public static class DesignTokens
{
    public const double WindowWidth = 420;
    public const double SectionSpacing = 16;
    public const double RowSpacing = 8;
    public const double Padding = 20;
    public const double StatusDotSize = 10;
    public const double LogHeight = 140;
    public const double CitiesEditorHeight = 120;
    public const double ProtocolRowSpacing = 2;
    public const double HeaderIconSize = 32;
    public const double MenuIconSize = 16;
    public const double AboutIconSize = 64;
    public const double AboutWidth = 300;
    public const double TitleFontSize = 20;
    public const double CaptionFontSize = 12;
    public const double LogFontSize = 12;
    public const double BusyIndicatorWidth = 48;
    public const double ReorderButtonWidth = 40;

    public static Thickness WindowPadding { get; } = new(Padding);

    public static Thickness SectionGap { get; } = new(0, SectionSpacing, 0, 0);

    public static Thickness RowGap { get; } = new(0, RowSpacing, 0, 0);

    public static Thickness InlineGap { get; } = new(RowSpacing, 0, 0, 0);

    public static Thickness HairlineThickness { get; } = new(1);

    public static Thickness OutputPadding { get; } = new(RowSpacing);

    public static Thickness CompactRowPadding { get; } = new(RowSpacing, ProtocolRowSpacing, RowSpacing, ProtocolRowSpacing);

    public static Thickness ProtocolRowMargin { get; } = new(0, ProtocolRowSpacing, 0, ProtocolRowSpacing);

    public static CornerRadius ControlCornerRadius { get; } = new(4);

    public static FontFamily MonospaceFontFamily { get; } = new("Cascadia Mono, Consolas");
}
