param (
    [string]$Milestone = "00"
)

$ErrorActionPreference = "Stop"

# Set environment variables for snapshot mode
$env:SERMON_MOCK = "1"
$env:SERMON_SNAPSHOT = "1"

# Construct artifacts path
# Using -foundation suffix as per convention seen in plans/milestones
$ArtifactsDir = "$Milestone-foundation"
$ArtifactsPath = Join-Path "artifacts" "ui" $ArtifactsDir

# Create artifacts directory if it doesn't exist
if (-not (Test-Path $ArtifactsPath)) {
    New-Item -ItemType Directory -Force -Path $ArtifactsPath | Out-Null
    Write-Host "Created artifacts directory: $ArtifactsPath" -ForegroundColor Green
}

# Print status and instructions
Write-Host ""
Write-Host "📸 SERMON SNAPSHOT RUNNER" -ForegroundColor Magenta
Write-Host "=========================" -ForegroundColor Magenta
Write-Host ""
Write-Host "Environment:" -ForegroundColor Cyan
Write-Host "  SERMON_MOCK     = $env:SERMON_MOCK"
Write-Host "  SERMON_SNAPSHOT = $env:SERMON_SNAPSHOT"
Write-Host "  Milestone       = $Milestone"
Write-Host "  Artifacts Path  = $ArtifactsPath"
Write-Host ""
Write-Host "Expected Screenshots:" -ForegroundColor Yellow
Write-Host "  - shell-library.png"
Write-Host "  - shell-now-playing.png"
Write-Host "  - shell-settings.png"
Write-Host ""
Write-Host "⚠️  DISPLAY REQUIREMENTS:" -ForegroundColor Red -BackgroundColor Black
Write-Host "  - Viewport: 1440 x 900"
Write-Host "  - Scale:    100% (Device Scale Factor 1.0)"
Write-Host ""
Write-Host "Starting application..." -ForegroundColor Green
Write-Host ""

# Run the application
cargo tauri dev
