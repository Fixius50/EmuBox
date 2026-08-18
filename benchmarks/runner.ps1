# EmuBox Benchmarking Suite Runner for Windows PowerShell
$ErrorActionPreference = "Stop"

Write-Host "========================================================" -ForegroundColor Cyan
Write-Host "   EMUBOX FRONTEND BENCHMARKING LAB (WINDOWS RUNNER)   " -ForegroundColor Cyan
Write-Host "========================================================" -ForegroundColor Cyan

$candidates = @("solid", "svelte", "react", "vue", "next", "astro")
$resultsDir = ".\benchmarks\results"

if (-not (Test-Path $resultsDir)) {
    New-Item -ItemType Directory -Path $resultsDir -Force | Out-Null
}

foreach ($framework in $candidates) {
    if (Test-Path $framework) {
        Write-Host "--------------------------------------------------------" -ForegroundColor Yellow
        Write-Host "Procesando $framework..." -ForegroundColor Yellow
        Push-Location $framework
        if (-not (Test-Path "node_modules")) {
            Write-Host "Instalando dependencias en $framework..."
            npm install --silent
        }
        Write-Host "Ejecutando build en $framework..."
        npm run build
        Pop-Location
    }
}

Write-Host "========================================================" -ForegroundColor Green
Write-Host "Procesamiento finalizado." -ForegroundColor Green
