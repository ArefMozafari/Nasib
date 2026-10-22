using System.Windows.Media;
using Microsoft.Win32;

namespace Nasib.App.Drawing;

/// <summary>
/// Picks the tray glyph color that shows up on the taskbar: black on a light taskbar, white on a dark one.
/// </summary>
public static class TaskbarTheme
{
    private const string PersonalizeKeyPath = @"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize";
    private const string SystemUsesLightThemeValueName = "SystemUsesLightTheme";
    private const int LightThemeValue = 1;

    /// <summary>
    /// Windows before the light taskbar had no such value and a dark taskbar, so a missing value means dark.
    /// </summary>
    public static bool IsLight
    {
        get
        {
            using var key = Registry.CurrentUser.OpenSubKey(PersonalizeKeyPath);
            return key?.GetValue(SystemUsesLightThemeValueName) is int value && value == LightThemeValue;
        }
    }

    public static Color GlyphColor => IsLight ? Colors.Black : Colors.White;
}
