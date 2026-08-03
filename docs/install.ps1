#Requires -Version 5.0  
<#  
    Lumesh Installation Script for Windows (PowerShell)  
    Usage:  
        powershell -c "irm https://raw.githubusercontent.com/superiums/lumesh/main/docs/install.ps1 | iex"  
#>  
  
$ErrorActionPreference = "Stop"  
  
$GithubRepo   = "superiums/lumesh"  
$AssetName    = "lume-x86_64-pc-windows-gnu.exe"  
$SeAssetName  = "lume-se-x86_64-pc-windows-gnu.exe"  
  
function Write-Info($msg)  { Write-Host $msg -ForegroundColor Cyan }  
function Write-Ok($msg)    { Write-Host $msg -ForegroundColor Green }  
function Write-Warn($msg)  { Write-Host $msg -ForegroundColor Yellow }  
function Write-Err($msg)   { Write-Host $msg -ForegroundColor Red }  
  
Write-Info "Lumesh Installation Script"  
Write-Info "=================================="  
  
# 1. 选择安装类型（Read-Host 在 irm | iex 下依然能正常读终端输入）  
Write-Warn "Choose installation type:"  
Write-Host "1) User installation (recommended) - installs to `$env:LOCALAPPDATA\lumesh"  
Write-Host "2) System installation (requires admin) - installs to C:\Program Files\lumesh"  
$choice = Read-Host "Enter choice (1-2) [1]"  
if ([string]::IsNullOrWhiteSpace($choice)) { $choice = "1" }  
  
switch ($choice) {  
    "2" {  
        $InstallDir = "C:\Program Files\lumesh"  
        $DocDir     = "C:\Program Files\lumesh"  
        $isAdmin = ([Security.Principal.WindowsIdentity]::GetCurrent()).Groups `  
            -contains "S-1-5-32-544"  
        if (-not $isAdmin) {  
            Write-Err "System installation requires an elevated (Administrator) PowerShell."  
            Write-Warn "Re-launch PowerShell as Administrator and re-run this script, or choose option 1."  
            exit 1  
        }  
        Write-Ok "System installation selected"  
    }  
    default {  
        $InstallDir = Join-Path $env:LOCALAPPDATA "lumesh"  
        $DocDir     = $InstallDir  
        Write-Ok "User installation selected"  
    }  
}  
  
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null  
New-Item -ItemType Directory -Force -Path $DocDir | Out-Null  
  
# 2. 获取最新版本信息  
Write-Info "Fetching latest release info..."  
$release = Invoke-RestMethod -Uri "https://api.github.com/repos/$GithubRepo/releases/latest" -Headers @{ "User-Agent" = "lumesh-installer" }  
$tag = $release.tag_name  
if (-not $tag) {  
    Write-Err "Failed to fetch latest version."  
    exit 1  
}  
Write-Ok "Latest version: $tag"  
  
function Get-AssetUrl($name) {  
    $asset = $release.assets | Where-Object { $_.name -eq $name }  
    if (-not $asset) {  
        Write-Err "Asset '$name' not found in release $tag"  
        exit 1  
    }  
    return $asset.browser_download_url  
}  
  
# 3. 下载主程序  
Write-Info "Downloading $AssetName ..."  
$exePath = Join-Path $InstallDir "lume.exe"  
Invoke-WebRequest -Uri (Get-AssetUrl $AssetName) -OutFile $exePath  
  
Write-Info "Downloading $SeAssetName ..."  
$sePath = Join-Path $InstallDir "lume-se.exe"  
Invoke-WebRequest -Uri (Get-AssetUrl $SeAssetName) -OutFile $sePath  
  
# 4. 下载并解压文档（doc.tgz）  
try {  
    Write-Info "Downloading documentation ..."  
    $docTgz = Join-Path $env:TEMP "doc.tgz"  
    Invoke-WebRequest -Uri (Get-AssetUrl "doc.tgz") -OutFile $docTgz  
    tar -xzf $docTgz -C $DocDir  
    Remove-Item $docTgz -Force  
} catch {  
    Write-Warn "Skipped documentation download (not critical): $($_.Exception.Message)"  
}  
  
# 5. 加入 PATH（当前用户或系统级）  
$target = if ($choice -eq "2") { "Machine" } else { "User" }  
$existingPath = [Environment]::GetEnvironmentVariable("PATH", $target)  
if ($existingPath -notlike "*$InstallDir*") {  
    Write-Info "Adding $InstallDir to $target PATH ..."  
    [Environment]::SetEnvironmentVariable("PATH", "$existingPath;$InstallDir", $target)  
    $env:PATH += ";$InstallDir"  
} else {  
    Write-Ok "$InstallDir is already in PATH"  
}  
  
Write-Host ""  
Write-Ok "Installation completed successfully!"  
Write-Host "Installation location: $InstallDir"  
Write-Host "To start using Lumesh (open a NEW terminal window first):"  
Write-Host "  Interactive shell: lume"  
Write-Host "  Script execution : lume-se script.lm"  
Write-Host "  Documentation    : $DocDir"  
Write-Host ""  
Write-Warn "Note: run 'lume' from Windows Terminal (wt) or cmder for best colorized prompt support."
