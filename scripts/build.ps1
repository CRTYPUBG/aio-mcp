param(
    [switch]$SkipNodeInstall
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$projectRoot = Resolve-Path "$PSScriptRoot\.."
$distDir = Join-Path $projectRoot "dist"
$reportsDir = Join-Path $distDir "reports"
$rustArtifactsDir = Join-Path $distDir "artifacts\rust"
$tsArtifactsDir = Join-Path $distDir "artifacts\typescript"
$docsDir = Join-Path $distDir "docs"
$schemasDir = Join-Path $distDir "schemas"

Write-Host "Preparing dist folder..."
if (Test-Path $distDir) {
    Remove-Item -Recurse -Force $distDir
}

New-Item -ItemType Directory -Path $reportsDir | Out-Null
New-Item -ItemType Directory -Path $rustArtifactsDir | Out-Null
New-Item -ItemType Directory -Path $tsArtifactsDir | Out-Null
New-Item -ItemType Directory -Path $docsDir | Out-Null
New-Item -ItemType Directory -Path $schemasDir | Out-Null

Push-Location $projectRoot
try {
    function Invoke-And-Log {
        param(
            [Parameter(Mandatory = $true)]
            [string]$Command,
            [Parameter(Mandatory = $true)]
            [string]$LogPath,
            [Parameter(Mandatory = $true)]
            [string]$FailureMessage
        )

        & cmd.exe /c "$Command 2>&1" | Tee-Object -FilePath $LogPath
        if ($LASTEXITCODE -ne 0) {
            throw $FailureMessage
        }
    }

    $buildSummary = [ordered]@{
        timestamp  = (Get-Date).ToString("o")
        rust       = [ordered]@{
            tests        = "not-run"
            releaseBuild = "not-run"
        }
        typescript = [ordered]@{
            available = $false
            install   = "skipped"
            build     = "skipped"
            typecheck = "skipped"
        }
    }

    Write-Host "Running Rust tests..."
    Invoke-And-Log -Command "cargo test --workspace" -LogPath (Join-Path $reportsDir "rust-test.log") -FailureMessage "Rust tests failed."
    $buildSummary.rust.tests = "passed"

    Write-Host "Running Rust release build..."
    Invoke-And-Log -Command "cargo build --workspace --release" -LogPath (Join-Path $reportsDir "rust-build.log") -FailureMessage "Rust release build failed."
    $buildSummary.rust.releaseBuild = "passed"

    $npmPath = Get-Command npm -ErrorAction SilentlyContinue
    if ($null -ne $npmPath) {
        $buildSummary.typescript.available = $true

        if (-not $SkipNodeInstall) {
            Write-Host "Installing Node workspace dependencies..."
            Invoke-And-Log -Command "npm install" -LogPath (Join-Path $reportsDir "npm-install.log") -FailureMessage "npm install failed."
            $buildSummary.typescript.install = "passed"
        }

        Write-Host "Running TypeScript build..."
        Invoke-And-Log -Command "npm run build" -LogPath (Join-Path $reportsDir "typescript-build.log") -FailureMessage "TypeScript build failed."
        $buildSummary.typescript.build = "passed"

        Write-Host "Running TypeScript typecheck..."
        Invoke-And-Log -Command "npm run typecheck" -LogPath (Join-Path $reportsDir "typescript-typecheck.log") -FailureMessage "TypeScript typecheck failed."
        $buildSummary.typescript.typecheck = "passed"
    }
    else {
        Write-Warning "npm is not available. TypeScript build and typecheck skipped."
    }

    Write-Host "Collecting Rust artifacts..."
    $releaseDepsPath = Join-Path $projectRoot "target\release\deps"
    if (Test-Path $releaseDepsPath) {
        Get-ChildItem $releaseDepsPath -File |
        Where-Object { $_.Name -match '^aio_' } |
        Copy-Item -Destination $rustArtifactsDir -Force
    }

    Write-Host "Collecting TypeScript artifacts..."
    $tsDistPaths = @(
        "apps\cli\dist",
        "apps\desktop\dist",
        "apps\web-dashboard\dist"
    )

    foreach ($relativePath in $tsDistPaths) {
        $source = Join-Path $projectRoot $relativePath
        if (Test-Path $source) {
            $dest = Join-Path $tsArtifactsDir ($relativePath -replace '[\\/]', '_')
            New-Item -ItemType Directory -Path $dest -Force | Out-Null
            Copy-Item -Path (Join-Path $source "*") -Destination $dest -Recurse -Force
        }
    }

    Write-Host "Collecting docs and schemas..."
    Copy-Item -Path (Join-Path $projectRoot "README.md") -Destination $docsDir -Force
    Copy-Item -Path (Join-Path $projectRoot "docs\architecture\*") -Destination $docsDir -Recurse -Force
    Copy-Item -Path (Join-Path $projectRoot "schemas\*") -Destination $schemasDir -Recurse -Force

    $manifestPath = Join-Path $distDir "manifest.json"
    $buildSummary | ConvertTo-Json -Depth 10 | Set-Content -Path $manifestPath -Encoding UTF8

    Write-Host "Build and packaging complete. Dist directory is ready." -ForegroundColor Green
}
finally {
    Pop-Location
}
