# Tests the engine, then publishes Nasib as one self-contained x64 executable and zips it.
#   ./build.ps1     writes build/Nasib-<version>-win-x64.zip next to this script
$ErrorActionPreference = 'Stop'
Set-Location $PSScriptRoot

$solution = 'Nasib.sln'
$appProject = 'src/Nasib.App/Nasib.App.csproj'
$runtime = 'win-x64'
$configuration = 'Release'
$buildDirectory = Join-Path $PSScriptRoot 'build'
$publishDirectory = Join-Path $buildDirectory 'publish'

# The version lives in Directory.Build.props, so the zip name and the About window always agree.
$version = ([xml](Get-Content 'Directory.Build.props')).Project.PropertyGroup.Version | Where-Object { $_ } | Select-Object -First 1
if (-not $version) { throw 'No <Version> found in Directory.Build.props.' }

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

$zipPath = Join-Path $buildDirectory "Nasib-$version-$runtime.zip"
Compress-Archive -Path (Join-Path $publishDirectory 'Nasib.exe'), (Join-Path $PSScriptRoot '..\LICENSE') -DestinationPath $zipPath -Force
Write-Host "built $zipPath"
