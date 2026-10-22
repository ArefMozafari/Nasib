using System;
using System.Runtime.InteropServices;

namespace Nasib.App;

/// <summary>
/// The two Win32 calls the tray needs that neither WPF nor Windows Forms wraps.
/// </summary>
internal static class NativeMethods
{
    [DllImport("user32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    internal static extern bool DestroyIcon(IntPtr handle);

    /// <summary>
    /// A menu opened from the tray closes on an outside click only when its window is in the foreground.
    /// </summary>
    [DllImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    internal static extern bool SetForegroundWindow(IntPtr windowHandle);
}
