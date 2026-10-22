using System;
using Microsoft.Win32;

namespace Nasib.App;

/// <summary>
/// Starts Nasib when the user signs in, through the per-user Run key. It starts in the tray, without the window.
/// </summary>
public static class LoginItem
{
    private const string RunKeyPath = @"Software\Microsoft\Windows\CurrentVersion\Run";
    private const string ValueName = "Nasib";

    public static bool IsEnabled
    {
        get
        {
            using var key = Registry.CurrentUser.OpenSubKey(RunKeyPath);
            return key?.GetValue(ValueName) is string;
        }
    }

    /// <summary>
    /// Points the Run value at this executable where it is now, so move it to where it will stay first.
    /// </summary>
    public static void SetEnabled(bool enabled)
    {
        using var key = Registry.CurrentUser.CreateSubKey(RunKeyPath, writable: true);
        if (enabled)
        {
            var executablePath = Environment.ProcessPath
                ?? throw new InvalidOperationException("The path of the running executable is unknown.");
            key.SetValue(ValueName, $"\"{executablePath}\" {App.BackgroundArgument}");
        }
        else
        {
            key.DeleteValue(ValueName, throwOnMissingValue: false);
        }
    }
}
