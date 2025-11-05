#!/usr/bin/env python3
"""
Monitor window events in real-time
Usage: python window_monitor.py
"""

import subprocess
import json
import sys
from datetime import datetime


def monitor_windows():
    """Monitor window creation, closing, and focus events"""
    print("Monitoring window events (Ctrl+C to stop)...", file=sys.stderr)
    
    try:
        proc = subprocess.Popen(
            ['twm', '--format', 'json', 'listen', '--events', 
             'window_created,window_closed,window_focused'],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            bufsize=1
        )
        
        for line in proc.stdout:
            try:
                event = json.loads(line.strip())
                timestamp = datetime.now().strftime("%H:%M:%S")
                event_name = event.get('name', 'unknown')
                event_data = event.get('data', {})
                
                if event_name == 'window_created':
                    title = event_data.get('title', 'Unknown')
                    workspace = event_data.get('workspace', '?')
                    print(f"[{timestamp}] ✓ Window Created: {title} (Workspace {workspace})")
                    
                elif event_name == 'window_closed':
                    hwnd = event_data.get('hwnd', 'Unknown')
                    print(f"[{timestamp}] ✗ Window Closed: HWND {hwnd}")
                    
                elif event_name == 'window_focused':
                    hwnd = event_data.get('hwnd', 'Unknown')
                    print(f"[{timestamp}] ◆ Window Focused: HWND {hwnd}")
                    
                else:
                    print(f"[{timestamp}] ? Unknown event: {event_name}")
                    
            except json.JSONDecodeError as e:
                print(f"Failed to parse event: {e}", file=sys.stderr)
                continue
                
    except KeyboardInterrupt:
        print("\nStopping monitor...", file=sys.stderr)
        proc.terminate()
        return 0
        
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        return 1


if __name__ == '__main__':
    sys.exit(monitor_windows())
