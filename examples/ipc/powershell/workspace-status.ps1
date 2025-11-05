#!/usr/bin/env pwsh
# Display current workspace status
# Usage: ./workspace-status.ps1

Write-Host "Workspace Status:" -ForegroundColor Cyan
Write-Host ""

try {
    $result = & twm --format json workspaces 2>&1
    
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Failed to get workspace information: $result" -ForegroundColor Red
        exit 1
    }
    
    $response = $result | ConvertFrom-Json
    
    if ($response.type -eq "success") {
        $workspaces = $response.data
        
        foreach ($ws in $workspaces) {
            $status = if ($ws.active) { "●" } else { "○" }
            $color = if ($ws.active) { "Green" } else { "Gray" }
            
            Write-Host "$status " -NoNewline -ForegroundColor $color
            Write-Host "Workspace $($ws.id): " -NoNewline -ForegroundColor White
            Write-Host "$($ws.name) " -NoNewline -ForegroundColor Cyan
            Write-Host "($($ws.window_count) windows, monitor $($ws.monitor))" -ForegroundColor Gray
        }
    }
    else {
        Write-Host "Error: $($response.message)" -ForegroundColor Red
        exit 1
    }
}
catch {
    Write-Host "Error: $_" -ForegroundColor Red
    exit 1
}
