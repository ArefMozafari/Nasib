# Tests and builds the shared Rust engine and the Windows app, then zips Nasib.exe with nasib-engine.exe.
#   ./build.ps1     writes build/Nasib-<version>-win-x64.zip next to this script
# Needs the .NET 9 SDK and Rust with the x86_64-pc-windows-msvc target.
$ErrorActionPreference = 'Stop'
Set-Location $PSScriptRoot

$solution = 'Nasib.sln'
$appProject = 'src/Nasib.App/Nasib.App.csproj'
$runtime = 'win-x64'
$engineTarget = 'x86_64-pc-windows-msvc'
$configuration = 'Release'
$engineDirectory = Join-Path $PSScriptRoot '..\Engine'
$buildDirectory = Join-Path $PSScriptRoot 'build'
$publishDirectory = Join-Path $buildDirectory 'publish'

# The version lives in Directory.Build.props, so the zip name and the About window always agree.
$version = ([xml](Get-Content 'Directory.Build.props')).Project.PropertyGroup.Version | Where-Object { $_ } | Select-Object -First 1
if (-not $version) { throw 'No <Version> found in Directory.Build.props.' }

# The engine is built first: the app's build copies nasib-engine.exe next to Nasib.exe.
Push-Location $engineDirectory
try {
    cargo test
    if ($LASTEXITCODE -ne 0) { throw 'Engine tests failed; nothing was published.' }
    cargo build --release --target $engineTarget
    if ($LASTEXITCODE -ne 0) { throw 'Engine build failed.' }
}
finally {
    Pop-Location
}

dotnet test $solution --configuration $configuration
if ($LASTEXITCODE -ne 0) { throw 'Tests failed; nothing was published.' }

if (Test-Path $publishDirectory) { Remove-Item $publishDirectory -Recurse -Force }
dotnet publish $appProject `
    --configuration $configuration `
    --runtime $runtime `
    --self-contained true `
    --output $publishDirectory `
    -p:PublishSingleFile=true `
    -p:IncludeNativeLibrariesForSelfExtract=true `
    -p:EnableCompressionInSingleFile=true `
    -p:DebugType=none
if ($LASTEXITCODE -ne 0) { throw 'Publish failed.' }

$appExecutable = Join-Path $publishDirectory 'Nasib.exe'
$engineExecutable = Join-Path $publishDirectory 'nasib-engine.exe'
if (-not (Test-Path $engineExecutable)) { throw 'nasib-engine.exe is missing from the published app.' }

$zipPath = Join-Path $buildDirectory "Nasib-$version-$runtime.zip"
Compress-Archive -Path $appExecutable, $engineExecutable, (Join-Path $PSScriptRoot '..\LICENSE') -DestinationPath $zipPath -Force
Write-Host "built $zipPath"
