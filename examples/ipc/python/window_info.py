#!/usr/bin/env python3
"""
Display information about the active window
Usage: python window_info.py
"""

import subprocess
import json
import sys


def get_active_window():
    """Get and display active window information"""
    try:
        result = subprocess.run(
            ['twm', '--format', 'json', 'active-window'],
            capture_output=True,
            text=True,
            check=False
        )
        
        if result.returncode != 0:
            print(f"Error: Failed to get active window information", file=sys.stderr)
            return 1
        
        response = json.loads(result.stdout)
        
        if response.get('type') != 'success':
            print(f"Error: {response.get('message', 'Unknown error')}", file=sys.stderr)
            return 1
        
        window = response.get('data', {})
        
        print("Active Window:")
        print()
        print(f"  HWND:       {window.get('hwnd', 'Unknown')}")
        print(f"  Title:      {window.get('title', 'Unknown')}")
        print(f"  Class:      {window.get('class', 'Unknown')}")
        print(f"  Process:    {window.get('process_name', 'Unknown')}")
        print(f"  Workspace:  {window.get('workspace', '?')}")
        print(f"  Monitor:    {window.get('monitor', '?')}")
        print(f"  State:      {window.get('state', 'Unknown')}")
        
        rect = window.get('rect', {})
        print(f"  Position:   {rect.get('x', 0)}, {rect.get('y', 0)}")
        print(f"  Size:       {rect.get('width', 0)}x{rect.get('height', 0)}")
        
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
    sys.exit(get_active_window())
