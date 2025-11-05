#!/usr/bin/env python3
"""
Automatically tile new windows as they are created
Usage: python auto_tiler.py
"""

import subprocess
import json
import sys
from datetime import datetime


def auto_tile():
    """Monitor window creation and automatically tile new windows"""
    print("Auto-tiling new windows (Ctrl+C to stop)...", file=sys.stderr)
    
    try:
        proc = subprocess.Popen(
            ['twm', '--format', 'json', 'listen', '--events', 'window_created'],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            bufsize=1
        )
        
        for line in proc.stdout:
            try:
                event = json.loads(line.strip())
                
                if event.get('name') == 'window_created':
                    event_data = event.get('data', {})
                    hwnd = event_data.get('hwnd')
                    title = event_data.get('title', 'Unknown')
                    workspace = event_data.get('workspace', '?')
                    
                    timestamp = datetime.now().strftime("%H:%M:%S")
                    print(f"[{timestamp}] New window: {title} (HWND: {hwnd}, Workspace: {workspace})")
                    
                    # Here you could add custom logic to:
                    # - Move window to specific workspace based on title/class
                    # - Toggle floating for specific applications
                    # - Set specific layout for certain workspaces
                    # Example:
                    # if 'notepad' in title.lower():
                    #     subprocess.run(['twm', 'move', hwnd, '2'])
                    
            except json.JSONDecodeError as e:
                print(f"Failed to parse event: {e}", file=sys.stderr)
                continue
                
    except KeyboardInterrupt:
        print("\nStopping auto-tiler...", file=sys.stderr)
        proc.terminate()
        return 0
        
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        return 1


if __name__ == '__main__':
    sys.exit(auto_tile())
