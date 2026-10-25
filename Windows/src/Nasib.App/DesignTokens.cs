using System.Windows;
using System.Windows.Media;

namespace Nasib.App;

/// <summary>
/// Sizes and spacing for the window, the tray menu and the About window, in one place.
/// </summary>
public static class DesignTokens
{
    public const double WindowWidth = 420;

    /// <summary>
    /// Settings is wider, so the two location lists sit side by side instead of stacking.
    /// </summary>
    public const double SettingsWindowWidth = 760;

    public const double LocationListsHeight = 300;
    public const double SmallArrowButtonSize = 24;
    public const double SmallArrowFontSize = 10;
    public const double GroupSpacing = 12;
    public const double LogLineHorizontalPadding = 4;
    public const double LogLineVerticalPadding = 2;
    public const double SectionSpacing = 16;
    public const double RowSpacing = 8;
    public const double Padding = 20;
    public const double StatusDotSize = 10;
    public const double LogHeight = 140;
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

    /// <summary>
    /// Wide enough for "Find Now", so the row does not shift when the button reads "Stop".
    /// </summary>
    public const double FindButtonMinWidth = 84;

    public static Thickness WindowPadding { get; } = new(Padding);

    public static Thickness SectionGap { get; } = new(0, SectionSpacing, 0, 0);

    public static Thickness RowGap { get; } = new(0, RowSpacing, 0, 0);

    public static Thickness InlineGap { get; } = new(RowSpacing, 0, 0, 0);

    public static Thickness HairlineThickness { get; } = new(1);

    public static Thickness OutputPadding { get; } = new(RowSpacing);

    public static Thickness CompactRowPadding { get; } = new(RowSpacing, ProtocolRowSpacing, RowSpacing, ProtocolRowSpacing);

    public static Thickness ProtocolRowMargin { get; } = new(0, ProtocolRowSpacing, 0, ProtocolRowSpacing);

    public static CornerRadius ControlCornerRadius { get; } = new(4);

    public static CornerRadius GroupCornerRadius { get; } = new(8);

    /// <summary>
    /// The space inside a Settings section, between its edge and its content.
    /// </summary>
    public static Thickness GroupPadding { get; } = new(GroupSpacing);

    public static Thickness NoPadding { get; } = new(0);

    /// <summary>
    /// Keeps a list's rows clear of the scroll bar, which the Fluent theme draws over the content.
    /// </summary>
    public static Thickness ScrollBarGutter { get; } = new(0, 0, GroupSpacing, 0);

    public static Thickness ArrowGap { get; } = new(ProtocolRowSpacing, 0, 0, 0);

    /// <summary>
    /// Lines the hint up with where the search box's own text starts.
    /// </summary>
    public static Thickness SearchHintMargin { get; } = new(GroupSpacing, 0, 0, 0);

    /// <summary>
    /// The space between the two location columns.
    /// </summary>
    public static GridLength ColumnGap { get; } = new(SectionSpacing);

    /// <summary>
    /// Space around each log line, so its stripe reads as a row.
    /// </summary>
    public static Thickness LogLinePadding { get; } = new(LogLineHorizontalPadding, LogLineVerticalPadding, LogLineHorizontalPadding, LogLineVerticalPadding);

    public static FontFamily MonospaceFontFamily { get; } = new("Cascadia Mono, Consolas");

    /// <summary>
    /// The color of the line that starts each stage of a search: the teal macOS uses, which reads on light and
    /// dark backgrounds alike. Fluent's attention brush follows the accent color, so it could look like an error.
    /// </summary>
    public static SolidColorBrush PhaseBrush { get; } = CreateFrozenBrush(Color.FromRgb(0x30, 0xB0, 0xC7));

    private static SolidColorBrush CreateFrozenBrush(Color color)
    {
        var brush = new SolidColorBrush(color);
        brush.Freeze();
        return brush;
    }
}
