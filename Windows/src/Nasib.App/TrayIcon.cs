using System;
using System.ComponentModel;
using System.Diagnostics;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Controls.Primitives;
using System.Windows.Data;
using System.Windows.Interop;
using Microsoft.Win32;
using Nasib.App.Drawing;
using Forms = System.Windows.Forms;

namespace Nasib.App;

/// <summary>
/// The waves in the notification area and the menu they open: the app header, the status line,
/// Open Nasib, Find Now, Update to (when a newer release exists), About Nasib and Quit Nasib.
/// </summary>
public sealed class TrayIcon : IDisposable
{
    private const string ApplicationName = "Nasib";
    private const string TooltipSeparator = " - ";

    /// <summary>
    /// Windows limits the notification-area tooltip to this many characters.
    /// </summary>
    private const int TooltipLimit = 127;

    /// <summary>
    /// Windows 10 and 11 ignore this and use the user's notification settings, but the call still needs one.
    /// </summary>
    private const int NotificationTimeoutMilliseconds = 5000;

    private readonly EngineController controller;
    private readonly UpdateChecker updateChecker;
    private readonly Window menuOwner;
    private readonly Action showMainWindow;
    private readonly Forms.NotifyIcon notifyIcon;
    private readonly ContextMenu menu;
    private System.Drawing.Icon? currentIcon;

    // The page the notification on screen opens when clicked; null opens the window instead.
    private Uri? notificationPage;

    public TrayIcon(EngineController controller, UpdateChecker updateChecker, Window menuOwner, Action showMainWindow, Action showAbout, Action quit)
    {
        this.controller = controller;
        this.updateChecker = updateChecker;
        this.menuOwner = menuOwner;
        this.showMainWindow = showMainWindow;
        menu = BuildMenu(showAbout, quit);
        notifyIcon = new Forms.NotifyIcon { Text = ApplicationName };
        notifyIcon.MouseUp += OnMouseUp;
        notifyIcon.BalloonTipClicked += (_, _) => OnNotificationClicked();
        notifyIcon.BalloonTipClosed += (_, _) => notificationPage = null;
        controller.PropertyChanged += OnControllerPropertyChanged;
        SystemEvents.UserPreferenceChanged += OnUserPreferenceChanged;
        UpdateIcon();
        notifyIcon.Visible = true;
    }

    public void ShowNotification(string title, string body)
    {
        ShowNotification(title, body, page: null);
    }

    /// <summary>
    /// Shows a notification that opens the given page when clicked, or the window when there is none.
    /// </summary>
    public void ShowNotification(string title, string body, Uri? page)
    {
        notificationPage = page;
        notifyIcon.ShowBalloonTip(NotificationTimeoutMilliseconds, title, body, Forms.ToolTipIcon.None);
    }

    public void Dispose()
    {
        SystemEvents.UserPreferenceChanged -= OnUserPreferenceChanged;
        controller.PropertyChanged -= OnControllerPropertyChanged;
        notifyIcon.Visible = false;
        notifyIcon.Dispose();
        currentIcon?.Dispose();
    }

    private ContextMenu BuildMenu(Action showAbout, Action quit)
    {
        // The header row also opens the window, so it reads as an enabled item rather than a greyed-out label.
        var header = new MenuItem
        {
            Header = ApplicationName,
            FontWeight = FontWeights.SemiBold,
            Icon = new Image
            {
                Source = AppIconDrawing.ImageSource,
                Width = DesignTokens.MenuIconSize,
                Height = DesignTokens.MenuIconSize,
            },
        };
        header.Click += (_, _) => showMainWindow();

        var statusLine = new MenuItem { IsEnabled = false };
        statusLine.SetBinding(HeaderedItemsControl.HeaderProperty, new Binding($"{nameof(EngineController.State)}.{nameof(ConnectionState.Title)}"));

        var openNasib = new MenuItem { Header = "Open Nasib" };
        openNasib.Click += (_, _) => showMainWindow();

        // Reads "Stop Search" while any find runs, and stops it.
        var findNow = new MenuItem();
        findNow.SetBinding(HeaderedItemsControl.HeaderProperty, new Binding(nameof(EngineController.FindMenuItemTitle)));
        findNow.SetBinding(UIElement.IsEnabledProperty, new Binding(nameof(EngineController.CanUseFindButton)));
        findNow.Click += (_, _) => controller.ToggleFind();

        // Shown only when a newer release exists; opens its page in the browser.
        var update = new MenuItem { DataContext = updateChecker };
        update.SetBinding(HeaderedItemsControl.HeaderProperty, new Binding(nameof(UpdateChecker.UpdateMenuItemTitle)));
        update.SetBinding(UIElement.VisibilityProperty, new Binding(nameof(UpdateChecker.HasUpdate)) { Converter = new BooleanToVisibilityConverter() });
        update.Click += (_, _) => updateChecker.OpenReleasePage();

        var about = new MenuItem { Header = "About Nasib" };
        about.Click += (_, _) => showAbout();

        var quitItem = new MenuItem { Header = "Quit Nasib" };
        quitItem.Click += (_, _) => quit();

        var contextMenu = new ContextMenu { DataContext = controller, Placement = PlacementMode.MousePoint };
        contextMenu.Items.Add(header);
        contextMenu.Items.Add(statusLine);
        contextMenu.Items.Add(new Separator());
        contextMenu.Items.Add(openNasib);
        contextMenu.Items.Add(findNow);
        contextMenu.Items.Add(new Separator());
        contextMenu.Items.Add(update);
        contextMenu.Items.Add(about);
        contextMenu.Items.Add(quitItem);
        return contextMenu;
    }

    /// <summary>
    /// As Windows apps do: a left click shows the window and a right click opens the menu.
    /// </summary>
    private void OnMouseUp(object? sender, Forms.MouseEventArgs eventArguments)
    {
        if (eventArguments.Button == Forms.MouseButtons.Left)
        {
            showMainWindow();
            return;
        }
        if (eventArguments.Button != Forms.MouseButtons.Right)
        {
            return;
        }
        var handle = new WindowInteropHelper(menuOwner).EnsureHandle();
        NativeMethods.SetForegroundWindow(handle);
        menu.IsOpen = true;
    }

    private void OnNotificationClicked()
    {
        if (notificationPage is null)
        {
            showMainWindow();
            return;
        }
        Process.Start(new ProcessStartInfo(notificationPage.AbsoluteUri) { UseShellExecute = true });
        notificationPage = null;
    }

    private void OnControllerPropertyChanged(object? sender, PropertyChangedEventArgs eventArguments)
    {
        if (eventArguments.PropertyName == nameof(EngineController.State))
        {
            UpdateIcon();
        }
    }

    private void OnUserPreferenceChanged(object? sender, UserPreferenceChangedEventArgs eventArguments)
    {
        // A taskbar switching between light and dark arrives as a General preference change.
        if (eventArguments.Category == UserPreferenceCategory.General)
        {
            menuOwner.Dispatcher.BeginInvoke(UpdateIcon);
        }
    }

    private void UpdateIcon()
    {
        var pixelSize = Forms.SystemInformation.SmallIconSize.Width;
        var bitmap = WaveGlyph.Render(controller.State.Glyph, TaskbarTheme.GlyphColor, pixelSize);
        var previousIcon = currentIcon;
        currentIcon = IconConversion.ToIcon(bitmap);
        notifyIcon.Icon = currentIcon;
        previousIcon?.Dispose();

        var tooltip = ApplicationName + TooltipSeparator + controller.State.Title;
        notifyIcon.Text = tooltip.Length > TooltipLimit ? tooltip[..TooltipLimit] : tooltip;
    }
}
