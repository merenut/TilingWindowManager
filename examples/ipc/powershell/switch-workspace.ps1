#!/usr/bin/env pwsh
# Switch to a workspace by ID
# Usage: ./switch-workspace.ps1 <workspace_id>

param(
    [Parameter(Mandatory=$true)]
    [int]$WorkspaceId
)

Write-Host "Switching to workspace $WorkspaceId..." -ForegroundColor Cyan

try {
    $result = & twm workspace $WorkspaceId 2>&1
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "Successfully switched to workspace $WorkspaceId" -ForegroundColor Green
    }
    else {
        Write-Host "Failed to switch workspace: $result" -ForegroundColor Red
        exit 1
    }
}
catch {
    Write-Host "Error: $_" -ForegroundColor Red
    exit 1
}
