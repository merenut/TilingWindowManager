#!/usr/bin/env python3
"""
Display current workspace status
Usage: python workspace_status.py
"""

import subprocess
import json
import sys


def get_workspaces():
    """Get and display workspace information"""
    try:
        result = subprocess.run(
            ['twm', '--format', 'json', 'workspaces'],
            capture_output=True,
            text=True,
            check=False
        )
        
        if result.returncode != 0:
            print(f"Error: Failed to get workspace information", file=sys.stderr)
            return 1
        
        response = json.loads(result.stdout)
        
        if response.get('type') != 'success':
            print(f"Error: {response.get('message', 'Unknown error')}", file=sys.stderr)
            return 1
        
        workspaces = response.get('data', [])
        
        print("Workspace Status:")
        print()
        
        for ws in workspaces:
            ws_id = ws.get('id', '?')
            name = ws.get('name', 'Unknown')
            monitor = ws.get('monitor', '?')
            window_count = ws.get('window_count', 0)
            active = ws.get('active', False)
            
            status = "●" if active else "○"
            
            print(f"{status} Workspace {ws_id}: {name} ({window_count} windows, monitor {monitor})")
        
        return 0
        
    except json.JSONDecodeError as e:
        print(f"Error parsing response: {e}", file=sys.stderr)
        return 1
        
    except FileNotFoundError:
        print("Error: 'twm' command not found. Is the CLI tool installed?", file=sys.stderr)
        return 1
        
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        return 1


if __name__ == '__main__':
    sys.exit(get_workspaces())
