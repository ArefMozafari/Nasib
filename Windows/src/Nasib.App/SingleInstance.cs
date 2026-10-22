using System;
using System.Threading;

namespace Nasib.App;

/// <summary>
/// Keeps one Nasib per signed-in user. A second launch asks the first to show its window, then exits.
/// </summary>
public sealed class SingleInstance : IDisposable
{
    private const string MutexName = @"Local\Nasib.SingleInstance";
    private const string ShowWindowEventName = @"Local\Nasib.ShowWindow";

    private readonly Mutex mutex;
    private readonly EventWaitHandle showWindowEvent;
    private readonly RegisteredWaitHandle registration;

    private SingleInstance(Mutex mutex, Action showWindowRequested)
    {
        this.mutex = mutex;
        showWindowEvent = new EventWaitHandle(false, EventResetMode.AutoReset, ShowWindowEventName);
        registration = ThreadPool.RegisterWaitForSingleObject(
            showWindowEvent,
            (_, _) => showWindowRequested(),
            state: null,
            Timeout.Infinite,
            executeOnlyOnce: false);
    }

    /// <summary>
    /// Returns the claim when this is the first instance, or null when another one is already running.
    /// The callback runs on a thread-pool thread.
    /// </summary>
    public static SingleInstance? TryClaim(Action showWindowRequested)
    {
        var mutex = new Mutex(initiallyOwned: true, MutexName, out var createdNew);
        if (!createdNew)
        {
            mutex.Dispose();
            return null;
        }
        return new SingleInstance(mutex, showWindowRequested);
    }

    public static void AskRunningInstanceToShowItsWindow()
    {
        if (EventWaitHandle.TryOpenExisting(ShowWindowEventName, out var showWindowEvent))
        {
            using (showWindowEvent)
            {
                showWindowEvent.Set();
            }
        }
    }

    public void Dispose()
    {
        registration.Unregister(null);
        showWindowEvent.Dispose();
        mutex.ReleaseMutex();
        mutex.Dispose();
    }
}
