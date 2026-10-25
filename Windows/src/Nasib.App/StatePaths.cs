using System;
using System.IO;

namespace Nasib.App;

/// <summary>
/// Where Nasib keeps its attempt log, the last pair that worked, and the saved settings. The engine reads
/// and writes the same folder; Engine/README.md lists what it keeps there.
/// </summary>
public sealed class StatePaths
{
    private const string ApplicationFolderName = "Nasib";
    private const string LogFileName = "log.csv";
    private const string ConfigurationFileName = "config.json";

    public StatePaths(string rootDirectory)
    {
        RootDirectory = rootDirectory;
    }

    public string RootDirectory { get; }

    public string LogFile => Path.Combine(RootDirectory, LogFileName);

    public string ConfigurationFile => Path.Combine(RootDirectory, ConfigurationFileName);

    /// <summary>
    /// The folder under %LOCALAPPDATA%, next to where Windscribe keeps its own files.
    /// </summary>
    public static StatePaths ForCurrentUser()
    {
        var localApplicationData = Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData);
        return new StatePaths(Path.Combine(localApplicationData, ApplicationFolderName));
    }

    public void EnsureExists()
    {
        Directory.CreateDirectory(RootDirectory);
    }
}
