#!/usr/bin/env pwsh
# Toggle between dwindle and master layouts
# Usage: ./toggle-layout.ps1

Write-Host "Toggling layout..." -ForegroundColor Cyan

try {
    # Get current config to check layout
    $configResult = & twm --format json config 2>&1
    
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Failed to get config: $configResult" -ForegroundColor Red
        exit 1
    }
    
    $config = ($configResult | ConvertFrom-Json).data
    $currentLayout = $config.current_layout
    
    # Toggle layout
    $newLayout = if ($currentLayout -eq "dwindle") { "master" } else { "dwindle" }
    
    Write-Host "Current layout: $currentLayout" -ForegroundColor Gray
    Write-Host "Switching to: $newLayout" -ForegroundColor Yellow
    
    $result = & twm layout $newLayout 2>&1
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "Successfully switched to $newLayout layout" -ForegroundColor Green
    }
    else {
        Write-Host "Failed to switch layout: $result" -ForegroundColor Red
        exit 1
    }
}
catch {
    Write-Host "Error: $_" -ForegroundColor Red
    exit 1
}
