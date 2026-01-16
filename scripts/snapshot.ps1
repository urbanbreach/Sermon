param (
    [string]$Milestone = "00"
)

$ErrorActionPreference = "Stop"

# Milestone to directory mapping
$MilestoneMap = @{
    "00" = "00-foundation"
    "01" = "01-library-db-scan"
    "02" = "02-playback-shared-now-playing"
    "03" = "03-wasapi-exclusive-bit-perfect"
}

# Milestone to expected screenshots mapping
$ScreenshotMap = @{
    "00" = @("shell-library.png", "shell-now-playing.png", "shell-settings.png")
    "01" = @("tracks-empty.png", "scanning.png", "tracks-populated.png")
    "02" = @("now-playing-idle.png", "now-playing-playing.png", "queue.png")
    "03" = @("settings-audio.png", "diagnostics-playing-44k.png", "diagnostics-playing-96k.png")
}

# Set environment variables for snapshot mode
$env:SERMON_MOCK = "1"
$env:SERMON_SNAPSHOT = "1"

# Get artifacts directory from mapping
$ArtifactsDir = $MilestoneMap[$Milestone]
if (-not $ArtifactsDir) {
    Write-Host "Unknown milestone: $Milestone" -ForegroundColor Red
    Write-Host "Available milestones: $($MilestoneMap.Keys -join ', ')" -ForegroundColor Yellow
    exit 1
}

$ArtifactsPath = Join-Path "artifacts" "ui" $ArtifactsDir

# Create artifacts directory if it doesn't exist
if (-not (Test-Path $ArtifactsPath)) {
    New-Item -ItemType Directory -Force -Path $ArtifactsPath | Out-Null
    Write-Host "Created artifacts directory: $ArtifactsPath" -ForegroundColor Green
}

# Get expected screenshots
$ExpectedScreenshots = $ScreenshotMap[$Milestone]

# Print status and instructions
Write-Host ""
Write-Host "📸 SERMON SNAPSHOT RUNNER" -ForegroundColor Magenta
Write-Host "==========================" -ForegroundColor Magenta
Write-Host ""
Write-Host "Environment:" -ForegroundColor Cyan
Write-Host "  SERMON_MOCK     = $env:SERMON_MOCK"
Write-Host "  SERMON_SNAPSHOT = $env:SERMON_SNAPSHOT"
Write-Host "  Milestone       = $Milestone"
Write-Host "  Artifacts Path  = $ArtifactsPath"
Write-Host ""
Write-Host "Expected Screenshots:" -ForegroundColor Yellow
foreach ($screenshot in $ExpectedScreenshots) {
    Write-Host "  - $screenshot"
}
Write-Host ""
Write-Host "⚠️  DISPLAY REQUIREMENTS:" -ForegroundColor Red -BackgroundColor Black
Write-Host "  - Viewport: 1440 x 900"
Write-Host "  - Scale:    100% (Device Scale Factor 1.0)"
Write-Host ""
Write-Host "Starting application..." -ForegroundColor Green
Write-Host ""

# Run the application
cargo tauri dev
