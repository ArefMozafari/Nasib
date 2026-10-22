namespace Nasib.Engine.Tests.Fakes;

/// <summary>
/// A throwaway folder under the system temp folder, deleted when the test ends.
/// </summary>
internal sealed class TemporaryDirectory : IDisposable
{
    public TemporaryDirectory()
    {
        Path = System.IO.Path.Combine(System.IO.Path.GetTempPath(), "nasib-tests-" + Guid.NewGuid().ToString("N"));
        Directory.CreateDirectory(Path);
    }

    public string Path { get; }

    public string Combine(string fileName) => System.IO.Path.Combine(Path, fileName);

    public void Dispose()
    {
        Directory.Delete(Path, recursive: true);
    }
}
