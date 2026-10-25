using System;
using System.Globalization;
using System.Windows;
using System.Windows.Data;

namespace Nasib.App;

/// <summary>
/// Visible for an empty or null string, collapsed otherwise. Shows a search box's hint until the user types.
/// </summary>
public sealed class VisibleWhenEmptyConverter : IValueConverter
{
    public object Convert(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        return string.IsNullOrEmpty(value as string) ? Visibility.Visible : Visibility.Collapsed;
    }

    public object ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        throw new NotSupportedException();
    }
}

/// <summary>
/// Visible for anything but null or an empty string. Shows a message only when there is one.
/// </summary>
public sealed class VisibleWhenPresentConverter : IValueConverter
{
    public object Convert(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        return value is null || (value is string text && text.Length == 0) ? Visibility.Collapsed : Visibility.Visible;
    }

    public object ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        throw new NotSupportedException();
    }
}
