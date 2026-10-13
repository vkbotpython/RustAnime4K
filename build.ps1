#Requires -Version 5.1
<#
.SYNOPSIS
  Локальная сборка Anime Player (Windows): проверки + release + dist-пакет.
.DESCRIPTION
  Повторяет шаги CI (ci.yml / release.yml) локально:
  fmt + clippy + test, затем cargo build --release [--features video],
  smoke-тест (--help) и сборка папки dist/anime-player (exe + libmpv-2.dll
  + shaders/images/docs + favicon + README + LICENSE).
  Для линковки фичи video используется готовый mpvlib-local/mpv.lib:
    $env:RUSTFLAGS="-L<repo>/mpvlib-local"
  libmpv-2.dll ищется в: -MpvDll, ./libmpv-2.dll, ./mpvlib-local/,
  ./target/release/. Нет DLL — положи её в одно из этих мест или
  используй -DownloadMpv (скачает mpv-dev и соберёт mpv.lib, нужны
  VS BuildTools с dumpbin/lib.exe и 7z в PATH).
.EXAMPLE
  .\build.ps1
  .\build.ps1 -NoVideo
  .\build.ps1 -SkipChecks -NoPackage
  .\build.ps1 -Zip
  .\build.ps1 -MpvDll C:\mpv\libmpv-2.dll
  .\build.ps1 -DownloadMpv
#>
[CmdletBinding()]
param(
  [switch]$NoVideo,
  [switch]$SkipChecks,
  [switch]$NoPackage,
  [switch]$Zip,
  [string]$MpvDll = "",
  [switch]$DownloadMpv
)

$ErrorActionPreference = 'Stop'
$RepoRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $RepoRoot

function Step([string]$msg) {
  Write-Host ""
  Write-Host "=== $msg ===" -ForegroundColor Cyan
}

function Find-MpvDll([string]$hint) {
  $candidates = @()
  if ($hint -ne "") { $candidates += $hint }
  $candidates += (Join-Path $RepoRoot 'libmpv-2.dll')
  $candidates += (Join-Path $RepoRoot 'mpvlib-local\libmpv-2.dll')
  $candidates += (Join-Path $RepoRoot 'target\release\libmpv-2.dll')
  # DLL от прошлой debug-сборки (тот же файл, годится и для release).
  $candidates += (Join-Path $RepoRoot 'target\debug\libmpv-2.dll')
  # Прошлая сборка/пакет (например, скачанный из CI Actions).
  $candidates += (Join-Path $RepoRoot 'dist\anime-player\libmpv-2.dll')
  # Установленный mpv-плеер и прочие каталоги из PATH.
  foreach ($dir in ($env:PATH -split ';')) {
    if (($dir -ne $null) -and ($dir -ne "")) {
      $candidates += (Join-Path $dir 'libmpv-2.dll')
    }
  }
  foreach ($c in $candidates) {
    if (($c -ne $null) -and ($c -ne "") -and (Test-Path $c)) {
      return (Resolve-Path $c).Path
    }
  }
  return $null
}

function Find-ToolDir {
  # dumpbin.exe/lib.exe из VS BuildTools (как в release.yml).
  $roots = @('C:/Program Files/Microsoft Visual Studio', 'C:/Program Files (x86)/Microsoft Visual Studio')
  foreach ($root in $roots) {
    if (-not (Test-Path $root)) { continue }
    $found = Get-ChildItem $root -Recurse -Filter dumpbin.exe -ErrorAction SilentlyContinue |
      Where-Object { $_.FullName -match 'Hostx64\\x64' } | Select-Object -First 1
    if ($found) { return $found.DirectoryName }
  }
  return $null
}

function Install-MpvDev {
  # Скачивает mpv-dev, достаёт libmpv-2.dll и генерирует mpvlib-local/mpv.lib.
  # Возвращает полный путь к DLL.
  $dlDir = Join-Path $RepoRoot '.mpv-dl'
  New-Item -ItemType Directory -Force $dlDir | Out-Null
  Step 'Скачивание mpv-dev (shinchiro/mpv-winbuild-cmake)'
  $rel = Invoke-RestMethod 'https://api.github.com/repos/shinchiro/mpv-winbuild-cmake/releases/latest'
  $asset = @($rel.assets | Where-Object { $_.name -match '^mpv-dev-x86_64-\d.*\.7z$' })[0]
  if (-not $asset) { throw 'libmpv archive not found in latest mpv release' }
  $sevenZ = Join-Path $dlDir 'mpv.7z'
  Invoke-WebRequest $asset.browser_download_url -OutFile $sevenZ
  if (-not (Get-Command 7z -ErrorAction SilentlyContinue)) {
    throw 'Для распаковки mpv-dev нужен 7z в PATH'
  }
  $outDir = Join-Path $dlDir 'mpv_dist'
  7z x $sevenZ "-o$outDir" 'libmpv-2.dll' -y | Out-Null
  $dll = Get-ChildItem $outDir -Recurse -Filter libmpv-2.dll | Select-Object -First 1
  if (-not $dll) { throw 'libmpv-2.dll not found in archive' }
  $tool = Find-ToolDir
  if (-not $tool) { throw 'dumpbin.exe (VS BuildTools) not found' }
  $names = & "$tool/dumpbin.exe" /exports $dll.FullName |
    Select-String '^\s+\d+\s+[0-9A-F]+\s+[0-9A-F]+\s+(\S+)' |
    ForEach-Object { $_.Matches.Groups[1].Value }
  if (-not $names) { throw 'no exports parsed from libmpv-2.dll' }
  $libDir = Join-Path $RepoRoot 'mpvlib-local'
  New-Item -ItemType Directory -Force $libDir | Out-Null
  $defFile = Join-Path $dlDir 'mpv.def'
  @('LIBRARY libmpv-2', 'EXPORTS') + $names | Out-File $defFile -Encoding ascii
  & "$tool/lib.exe" "/def:$defFile" "/out:$libDir/mpv.lib" '/machine:x64'
  if ($LASTEXITCODE -ne 0) { throw 'lib.exe failed' }
  return $dll.FullName
}

# --- 0. Инструменты ---------------------------------------------------------
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
  throw 'cargo не найден в PATH. Установи Rust: https://rustup.rs'
}

$Video = -not $NoVideo
$LibDir = Join-Path $RepoRoot 'mpvlib-local'
$MpvLib = Join-Path $LibDir 'mpv.lib'
$DllPath = $null

if ($Video) {
  if (-not (Test-Path $MpvLib)) {
    if ($DownloadMpv) {
      $DllPath = Install-MpvDev
    } else {
      throw 'mpvlib-local/mpv.lib не найден. Собери его через .\build.ps1 -DownloadMpv (нужны VS BuildTools + 7z) или собери без video: .\build.ps1 -NoVideo'
    }
  }
  if (-not $DllPath) { $DllPath = Find-MpvDll $MpvDll }
  if (-not $DllPath -and $DownloadMpv) { $DllPath = Install-MpvDev }
  if (-not $DllPath) {
    throw 'libmpv-2.dll не найдена (проверены корень репо, mpvlib-local, target/release, dist и PATH). Скопируй её из прошлой сборки в корень репо, укажи -MpvDll <путь> или используй -DownloadMpv'
  }
  Write-Host "mpv.lib : $MpvLib"
  Write-Host "mpv dll : $DllPath"
  # Однобуквенная форма -L<путь> — один токен, переживает пробелы в пути.
  $env:RUSTFLAGS = ("$env:RUSTFLAGS -L$LibDir").Trim()
}

# --- 1. Проверки (как lint-test в CI) ---------------------------------------
if (-not $SkipChecks) {
  Step 'cargo fmt --check'
  cargo fmt --all -- --check
  Step 'cargo clippy'
  cargo clippy --all-targets -- -D warnings
  if ($Video) {
    Step 'cargo clippy --features video'
    cargo clippy --all-targets --features video -- -D warnings
  }
  Step 'cargo test'
  cargo test
  if ($Video) {
    Step 'cargo test --features video'
    cargo test --features video
  }
} else {
  Write-Host 'Проверки пропущены (-SkipChecks)'
}

# --- 2. Release-сборка -------------------------------------------------------
Step 'cargo build --release'
if ($Video) {
  cargo build --release --features video
} else {
  cargo build --release
}

$Exe = Join-Path $RepoRoot 'target\release\anime-player.exe'
if (-not (Test-Path $Exe)) { throw "Сборка не дала $Exe" }

# DLL рядом с exe, чтобы прошёл smoke-тест и запуск (как в release.yml).
if ($Video) {
  $DllDest = Join-Path $RepoRoot 'target\release\libmpv-2.dll'
  $same = [string]::Equals(
    [IO.Path]::GetFullPath($DllPath).TrimEnd('\', '/'),
    [IO.Path]::GetFullPath($DllDest).TrimEnd('\', '/'),
    [System.StringComparison]::OrdinalIgnoreCase)
  if (-not $same) {
    Copy-Item $DllPath $DllDest -Force
  }
}

# --- 3. Smoke-тест -----------------------------------------------------------
Step 'smoke-test: anime-player --help'
& $Exe --help | Out-Null
if ($LASTEXITCODE -ne 0) { throw 'smoke-test провален (не грузится libmpv-2.dll?)' }

# --- 4. Dist-пакет (раскладка 1:1 с release.yml) ------------------------------
if (-not $NoPackage) {
  Step 'Сборка dist/anime-player'
  $Dist = Join-Path $RepoRoot 'dist\anime-player'
  if (Test-Path (Join-Path $RepoRoot 'dist')) {
    Remove-Item (Join-Path $RepoRoot 'dist') -Recurse -Force
  }
  New-Item -ItemType Directory -Force $Dist | Out-Null
  Copy-Item $Exe $Dist/
  if ($Video) {
    Copy-Item (Join-Path $RepoRoot 'target\release\libmpv-2.dll') $Dist/
  }
  Copy-Item (Join-Path $RepoRoot 'assets\shaders') (Join-Path $Dist 'shaders') -Recurse
  Copy-Item (Join-Path $RepoRoot 'assets\images') (Join-Path $Dist 'images') -Recurse
  Copy-Item (Join-Path $RepoRoot 'assets\docs') (Join-Path $Dist 'docs') -Recurse
  Copy-Item (Join-Path $RepoRoot 'assets\favicon.ico') $Dist/
  Copy-Item (Join-Path $RepoRoot 'README.md'), (Join-Path $RepoRoot 'LICENSE') $Dist/
  Write-Host "Готово: $Dist"
  if ($Zip) {
    $ZipPath = Join-Path $RepoRoot 'anime-player-windows-x86_64.zip'
    if (Test-Path $ZipPath) { Remove-Item $ZipPath -Force }
    Compress-Archive -Path $Dist -DestinationPath $ZipPath
    Write-Host "Архив: $ZipPath"
  }
} else {
  Write-Host "Пакет пропущен (-NoPackage). Бинарь: $Exe"
}

Write-Host ''
Write-Host 'Сборка завершена успешно.' -ForegroundColor Green
