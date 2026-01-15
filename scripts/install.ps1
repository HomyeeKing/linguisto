$ErrorActionPreference = "Stop"

$repo = "HomyeeKing/linguist"

# 获取版本：优先使用环境变量，其次自动从 GitHub Releases latest 获取
if ($env:LINGUISTO_VERSION) {
  $version = $env:LINGUISTO_VERSION
} else {
  Write-Host "Fetching latest version from GitHub releases..."
  $latest = Invoke-WebRequest -Uri "https://api.github.com/repos/$repo/releases/latest" -UseBasicParsing | Select-Object -ExpandProperty Content | ConvertFrom-Json
  if (-not $latest.tag_name) {
    Write-Error "无法从 GitHub 获取最新版本号，请设置环境变量 LINGUISTO_VERSION 后重试"
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
  Write-Error "解压后未找到 linguist.exe"
}

$targetPath = Join-Path $installDir "linguist.exe"
Copy-Item $exePath $targetPath -Force

Write-Host "linguist 已安装到 $targetPath"

$userPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($userPath -notlike "*$installDir*") {
  Write-Host "注意：$installDir 尚未在 PATH 中。"
  Write-Host "你可以在 PowerShell 中执行如下命令将其加入 PATH："
  Write-Host "  [Environment]::SetEnvironmentVariable(\"PATH\", \"$installDir;\" + [Environment]::GetEnvironmentVariable(\"PATH\", \"User\"), \"User\")"
  Write-Host "然后重新打开终端后即可使用 linguist。"
} else {
  Write-Host "你可以直接运行: linguist"
}
