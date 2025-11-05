#!/usr/bin/env pwsh
# Monitor window events in real-time
# Usage: ./monitor-windows.ps1

Write-Host "Monitoring window events (Ctrl+C to stop)..." -ForegroundColor Cyan

$events = "window_created,window_closed,window_focused"

try {
    & twm listen --events $events | ForEach-Object {
        $event = $_ | ConvertFrom-Json
        
        $timestamp = Get-Date -Format "HH:mm:ss"
        $eventType = $event.name
        
        switch ($eventType) {
            "window_created" {
                $title = $event.data.title
                $workspace = $event.data.workspace
                Write-Host "[$timestamp] " -NoNewline -ForegroundColor Gray
                Write-Host "Window Created: " -NoNewline -ForegroundColor Green
                Write-Host "$title (Workspace $workspace)" -ForegroundColor White
            }
            "window_closed" {
                $hwnd = $event.data.hwnd
                Write-Host "[$timestamp] " -NoNewline -ForegroundColor Gray
                Write-Host "Window Closed: " -NoNewline -ForegroundColor Red
                Write-Host "HWND $hwnd" -ForegroundColor White
            }
            "window_focused" {
                $hwnd = $event.data.hwnd
                Write-Host "[$timestamp] " -NoNewline -ForegroundColor Gray
                Write-Host "Window Focused: " -NoNewline -ForegroundColor Yellow
                Write-Host "HWND $hwnd" -ForegroundColor White
            }
            default {
                Write-Host "[$timestamp] Unknown event: $eventType" -ForegroundColor Magenta
            }
        }
    }
}
catch {
    Write-Host "Error: $_" -ForegroundColor Red
    exit 1
}
