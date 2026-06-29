$ErrorActionPreference = "Stop"

Write-Host "Running Rust checks..."
Push-Location "$PSScriptRoot\.."
try {
    cargo check --workspace

    $npmPath = Get-Command npm -ErrorAction SilentlyContinue
    if ($null -eq $npmPath) {
        Write-Warning "npm was not found on PATH. Skipping TypeScript checks for this machine."
    }
    else {
        Write-Host "Running Node checks..."
        npm run typecheck
    }
}
finally {
    Pop-Location
}
