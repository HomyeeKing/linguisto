$ErrorActionPreference = "Stop"

$repo = "HomyeeKing/linguist"

# Get version: prefer env var, otherwise fetch from GitHub Releases latest
if ($env:LINGUISTO_VERSION) {
  $version = $env:LINGUISTO_VERSION
} else {
  Write-Host "Fetching latest version from GitHub releases..."
  $latest = Invoke-WebRequest -Uri "https://api.github.com/repos/$repo/releases/latest" -UseBasicParsing | Select-Object -ExpandProperty Content | ConvertFrom-Json
  if (-not $latest.tag_name) {
    Write-Error "Failed to fetch latest version from GitHub, please set LINGUISTO_VERSION manually and retry."
  }
  $version = $latest.tag_name
}

$zipName = "linguist-$version-x86_64-pc-windows-msvc.zip"
$baseUrl = "https://github.com/$repo/releases/download/$version"
$url = "$baseUrl/$zipName"

$installDir = Join-Path $env:USERPROFILE ".linguist\bin"
New-Item -ItemType Directory -Path $installDir -Force | Out-Null

$tmp = New-Item -ItemType Directory -Path ([IO.Path]::Combine([IO.Path]::GetTempPath(), "linguist-install-" + [guid]::NewGuid()))
$zipPath = Join-Path $tmp.FullName $zipName

Write-Host "Downloading $url ..."
Invoke-WebRequest -Uri $url -OutFile $zipPath

Add-Type -AssemblyName System.IO.Compression.FileSystem
[IO.Compression.ZipFile]::ExtractToDirectory($zipPath, $tmp.FullName, $true)

$exePath = Join-Path $tmp.FullName "linguist.exe"
if (-not (Test-Path $exePath)) {
  Write-Error "linguist.exe not found after extraction."
}

$targetPath = Join-Path $installDir "linguist.exe"
Copy-Item $exePath $targetPath -Force

Write-Host "linguist installed to $targetPath"

$userPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($userPath -notlike "*$installDir*") {
  Write-Host "Note: $installDir is not in your PATH."
  Write-Host "You can add it with the following command in PowerShell:"
  Write-Host "  [Environment]::SetEnvironmentVariable(\"PATH\", \"$installDir;\" + [Environment]::GetEnvironmentVariable(\"PATH\", \"User\"), \"User\")"
  Write-Host "Then reopen your terminal and run 'linguist'."
} else {
  Write-Host "You can now run: linguist"
}
