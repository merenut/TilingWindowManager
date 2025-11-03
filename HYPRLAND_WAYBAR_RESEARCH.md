# Hyprland and Waybar Feature Parity Research
## Comprehensive Documentation for Windows-Based Tiling Window Manager Implementation

**Document Version:** 1.0  
**Last Updated:** November 2025  
**Target Platform:** Windows (using Rust)  
**Source Systems:** Hyprland (Wayland) + Waybar

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Part 1: Hyprland Core Features](#part-1-hyprland-core-features)
3. [Part 2: Waybar Features & Design](#part-2-waybar-features--design)
4. [Part 3: Design Philosophy & User Experience](#part-3-design-philosophy--user-experience)
5. [Part 4: Windows Implementation Considerations](#part-4-windows-implementation-considerations)
6. [Feature Priority Matrix](#feature-priority-matrix)
7. [Windows Implementation Roadmap](#windows-implementation-roadmap)
8. [Reference Materials](#reference-materials)

---

## Executive Summary

This document provides comprehensive research on **Hyprland** (a dynamic tiling Wayland compositor) and **Waybar** (a customizable status bar) to facilitate building a feature-complete Windows-based tiling window manager using Rust. 

**Key Findings:**
- Hyprland uses a binary tree-based dynamic tiling system with the "dwindle" layout
- IPC is socket-based with JSON output capabilities  
- Waybar uses JSON config + CSS theming for maximum flexibility
- Windows implementation via DWM API is feasible with automation layer
- Existing Rust project **komorebi** provides implementation patterns

**Implementation Feasibility:**
- **Direct Ports**: ~60% of features can be directly implemented
- **Adaptations Required**: ~30% need Windows-specific modifications
- **Windows Enhancements**: ~10% can be improved beyond Linux counterpart

---

## Part 1: Hyprland Core Features

### 1. Window Management

#### 1.1 Tiling Algorithms & Layouts

##### Dwindle Layout (Primary)

**Description:**
- Binary space partitioning (BSP) algorithm inspired by bspwm
- Dynamic tiling with automatic window placement
- Each workspace maintains a binary tree structure

**Technical Implementation:**
```
Algorithm:
- Each node in tree = window or container
- Split direction determined by container aspect ratio:
  - If width > height: vertical split (side-by-side)
  - If height > width: horizontal split (stacked)
  
Pseudocode:
function split_node(node):
    if node.width > node.height:
        left_child = Node(x=node.x, y=node.y, w=node.w/2, h=node.h)
        right_child = Node(x=node.x + node.w/2, y=node.y, w=node.w/2, h=node.h)
    else:
        top_child = Node(x=node.x, y=node.y, w=node.w, h=node.h/2)
        bottom_child = Node(x=node.x, y=node.y + node.h/2, w=node.w, h=node.h/2)
```

**Configuration Options:**
```ini
dwindle {
    pseudotile = yes          # Enable pseudotiling (window keeps size)
    preserve_split = yes      # Keep split direction permanent
    force_split = 0           # 0=auto, 1=always left, 2=always right
    no_gaps_when_only = no    # Remove gaps for single window
    smart_split = yes         # Auto-determine best split direction
    smart_resizing = yes      # Resize windows intelligently
}
```

##### Master Layout

**Description:**
- One master window + stack of secondary windows
- Master occupies larger portion (typically left/top)
- New windows added to stack

**Configuration:**
```ini
master {
    new_is_master = false       # New window becomes master
    new_on_top = false          # New windows on top of stack
    mfact = 0.55               # Master area factor (0.0-1.0)
    orientation = left         # left, right, top, bottom, center
    inherit_fullscreen = true  # Inherit fullscreen state
}
```

##### Floating Windows

**Features:**
- Windows exempt from tiling grid
- Manual positioning and sizing
- Toggle per-window via keybind or rule
- Useful for dialogs, pop-ups, utility windows

**Keybind Example:**
```ini
bind = SUPER, V, togglefloating
bind = SUPER, P, pseudo        # Toggle pseudotiling
```

##### Fullscreen Modes

**Three fullscreen types:**
1. **Normal (0)**: Fullscreen in tiling space
2. **Maximize (1)**: Fullscreen without hiding panels  
3. **Monocle (2)**: True fullscreen, hides everything

**Usage:**
```ini
bind = SUPER, F, fullscreen, 0
bind = SUPER_SHIFT, F, fullscreen, 1
```

#### 1.2 Window Stacking & Z-Order

**Features:**
- Automatic z-order management
- Floating windows above tiled
- Focus-follows-mouse optional
- Layer system for special windows

### 2. Workspaces

**Core Concepts:**
- Virtual desktops for window organization
- Independent window sets per workspace
- Persistent across sessions (optional)
- Per-monitor or shared models supported

**Features:**
- **Dynamic Creation**: Workspaces created on-demand
- **Naming**: Custom names or numbers (1-10 default)
- **Persistence**: Remember state between sessions
- **Multi-Monitor**: Separate workspaces per monitor or shared

**Configuration Examples:**
```ini
# Workspace bindings
bind = SUPER, 1, workspace, 1
bind = SUPER, 2, workspace, 2
bind = SUPER_SHIFT, 1, movetoworkspace, 1

# Special workspace (scratchpad)
bind = SUPER, S, togglespecialworkspace
bind = SUPER_SHIFT, S, movetoworkspace, special

# Workspace rules
workspace = 1, monitor:DP-1, default:true
workspace = 2, monitor:DP-1
workspace = 6, monitor:HDMI-A-1, default:true
```

### 3. Window Rules & Automation

#### 3.1 Window Rules (windowrulev2)

**Syntax:**
```ini
windowrulev2 = RULE, CRITERIA
```

**Available Rules:**
- `float` - Make window floating
- `tile` - Force tiling
- `fullscreen` - Start fullscreen
- `workspace` - Assign to workspace
- `monitor` - Assign to monitor
- `opacity` - Set transparency (active, inactive)
- `size` - Set window size
- `move` - Set window position  
- `center` - Center window
- `pin` - Pin window (visible on all workspaces)
- `noinitialfocus` - Don't focus on open
- `nofocus` - Never focus
- `noblur` - Disable blur
- `noshadow` - Disable shadow
- `maximize` - Maximize window
- `idleinhibit` - Prevent idle/screensaver

**Matching Criteria:**
- `class:^(regex)$` - Window class
- `title:^(regex)$` - Window title
- `initialClass` - Class at creation
- `initialTitle` - Title at creation

**Examples:**
```ini
# Float all Steam windows
windowrulev2 = float, class:^(steam|Steam)$

# Firefox on workspace 2, with opacity
windowrulev2 = workspace 2, class:^(firefox)$
windowrulev2 = opacity 0.95 0.85, class:^(firefox)$

# Center file picker dialogs
windowrulev2 = center, title:^(Open File).*$
windowrulev2 = size 800 600, title:^(Open File).*$

# Pin picture-in-picture
windowrulev2 = pin, title:^(Picture(-| )in(-| )[Pp]icture)$
windowrulev2 = float, title:^(Picture(-| )in(-| )[Pp]icture)$
```

#### 3.2 Layer Rules

**For special windows:**
```ini
layerrule = blur, waybar
layerrule = ignorezero, waybar
```

### 4. Input Handling

#### 4.1 Keybindings

**Bind Types:**
- `bind` - Standard key press
- `bindr` - On key release
- `binde` - Repeat while held
- `bindm` - Mouse binding

**Syntax:**
```ini
bind = MODIFIERS, KEY, DISPATCHER, PARAMS
```

**Modifiers:**
- `SUPER` (Windows key)
- `ALT`
- `SHIFT`
- `CTRL`
- Combinations: `SUPER_SHIFT`, `CTRL_ALT`, etc.

**Common Dispatchers:**
```ini
# Window management
bind = SUPER, Q, killactive
bind = SUPER, V, togglefloating
bind = SUPER, F, fullscreen
bind = SUPER, P, pseudo

# Focus movement
bind = SUPER, left, movefocus, l
bind = SUPER, right, movefocus, r
bind = SUPER, up, movefocus, u
bind = SUPER, down, movefocus, d

# Window movement
bind = SUPER_SHIFT, left, movewindow, l
bind = SUPER_SHIFT, right, movewindow, r

# Workspace switching
bind = SUPER, 1, workspace, 1
bind = SUPER, 2, workspace, 2

# Move to workspace
bind = SUPER_SHIFT, 1, movetoworkspace, 1

# Application launching
bind = SUPER, T, exec, kitty
bind = SUPER, E, exec, thunar
bind = SUPER, R, exec, wofi --show drun
```

#### 4.2 Mouse Bindings

```ini
# Move/resize windows with mouse
bindm = SUPER, mouse:272, movewindow
bindm = SUPER, mouse:273, resizewindow
```

#### 4.3 Submaps (Modes)

**Create mode-based keybindings:**
```ini
# Resize mode
bind = SUPER, R, submap, resize
submap = resize
binde = , right, resizeactive, 10 0
binde = , left, resizeactive, -10 0
binde = , up, resizeactive, 0 -10
binde = , down, resizeactive, 0 10
bind = , escape, submap, reset
submap = reset
```

### 5. Animations & Visual Effects

#### 5.1 Animation System

**Architecture:**
- Hierarchical animation tree
- Custom bezier curves for easing
- Per-animation configuration
- Performance-tunable

**Syntax:**
```ini
animation = NAME, ONOFF, SPEED, CURVE, STYLE
bezier = NAME, X0, Y0, X1, Y1
```

**Configuration Example:**
```ini
animations {
    enabled = yes
    
    # Define bezier curves
    bezier = myBezier, 0.05, 0.9, 0.1, 1.05
    bezier = linear, 0.0, 0.0, 1.0, 1.0
    bezier = fastSwitch, 0.20, 0.80, 0.80, 1.00
    
    # Window animations
    animation = windows, 1, 7, myBezier
    animation = windowsIn, 1, 7, myBezier, slide
    animation = windowsOut, 1, 7, default, popin 80%
    animation = windowsMove, 1, 7, default
    
    # Fade animations
    animation = fade, 1, 7, default
    animation = fadeIn, 1, 7, default
    animation = fadeOut, 1, 7, default
    animation = fadeSwitch, 1, 7, default
    animation = fadeShadow, 1, 7, default
    animation = fadeDim, 1, 7, default
    
    # Workspace animations
    animation = workspaces, 1, 6, fastSwitch, slide
    animation = specialWorkspace, 1, 6, default, slidevert
    
    # Border animation
    animation = border, 1, 10, default
    animation = borderangle, 1, 8, default
}
```

**Popular Bezier Presets:**
```ini
# Material Design 3
bezier = md3_standard, 0.2, 0.0, 0, 1.0
bezier = md3_decel, 0.05, 0.7, 0.1, 1.0
bezier = md3_accel, 0.3, 0.0, 0.8, 0.15

# Easing functions
bezier = easeInSine, 0.12, 0, 0.39, 0
bezier = easeOutSine, 0.61, 1, 0.88, 1
bezier = easeInOutSine, 0.37, 0, 0.63, 1
```

#### 5.2 Performance Optimization

**For low-end hardware:**
```ini
animations {
    enabled = yes
    
    # Faster, simpler animations
    bezier = simple, 0.0, 0.0, 1.0, 1.0
    animation = windows, 1, 3, simple
    animation = fade, 1, 2, simple
    animation = workspaces, 1, 4, simple
}

decoration {
    shadow {
        enabled = false
    }
    blur {
        enabled = false
    }
}
```

### 6. Decorations & Appearance

**Configuration:**
```ini
decoration {
    # Window rounding
    rounding = 10
    
    # Opacity
    active_opacity = 1.0
    inactive_opacity = 0.90
    fullscreen_opacity = 1.0
    
    # Blur
    blur {
        enabled = true
        size = 3
        passes = 1
        ignore_opacity = false
        new_optimizations = true
        xray = false
        noise = 0.0117
        contrast = 0.8916
        brightness = 0.8172
        vibrancy = 0.1696
        vibrancy_darkness = 0.0
    }
    
    # Shadows
    shadow {
        enabled = true
        range = 4
        render_power = 3
        color = rgba(1a1a1aee)
        offset = 0 0
    }
    
    # Dim inactive
    dim_inactive = false
    dim_strength = 0.5
    dim_special = 0.2
    dim_around = 0.4
}
```

**Window Borders:**
```ini
general {
    border_size = 2
    no_border_on_floating = false
    
    # Border colors
    col.active_border = rgba(33ccffee) rgba(00ff99ee) 45deg
    col.inactive_border = rgba(595959aa)
    
    # Border gradients
    col.active_border = rgb(ca9ee6) rgb(f2d5cf) 45deg
}
```

**Gaps:**
```ini
general {
    gaps_in = 5
    gaps_out = 20
    gaps_workspaces = 0
}
```

### 7. Multi-Monitor Support

#### 7.1 Monitor Configuration

**Syntax:**
```ini
monitor = NAME, RESOLUTION@REFRESH, POSITION, SCALE
```

**Examples:**
```ini
# By name
monitor = DP-1, 2560x1440@144, 0x0, 1
monitor = HDMI-A-1, 1920x1080@60, 2560x0, 1

# By description
monitor = desc:LG Electronics 34GL750, 2560x1080@144, 0x0, 1

# Auto configuration
monitor = , preferred, auto, 1

# Disable monitor
monitor = HDMI-A-2, disable
```

#### 7.2 Per-Monitor Workspaces

**Binding workspaces to monitors:**
```ini
workspace = 1, monitor:DP-1, default:true
workspace = 2, monitor:DP-1
workspace = 3, monitor:DP-1
workspace = 6, monitor:HDMI-A-1, default:true
workspace = 7, monitor:HDMI-A-1
```

#### 7.3 Monitor Commands

```bash
# List monitors
hyprctl monitors

# Move workspace to monitor
hyprctl dispatch moveworkspacetomonitor 1 HDMI-A-1

# Focus monitor
hyprctl dispatch focusmonitor HDMI-A-1
```

### 8. IPC & Control

#### 8.1 hyprctl Command Structure

**Socket Location:**
- `$XDG_RUNTIME_DIR/hypr/$HYPRLAND_INSTANCE_SIGNATURE/.socket.sock`
- `$XDG_RUNTIME_DIR/hypr/$HYPRLAND_INSTANCE_SIGNATURE/.socket2.sock` (events)

**Command Categories:**

1. **Query Commands** (get information):
```bash
hyprctl activewindow      # Get active window info
hyprctl clients           # List all windows
hyprctl workspaces        # List workspaces
hyprctl monitors          # List monitors
hyprctl devices           # List input devices
hyprctl binds             # List keybindings
hyprctl animations        # List animation config
hyprctl version           # Hyprland version
```

2. **Dispatch Commands** (perform actions):
```bash
hyprctl dispatch exec kitty                    # Launch app
hyprctl dispatch killactive                    # Close window
hyprctl dispatch workspace 2                   # Switch workspace
hyprctl dispatch movetoworkspace 3             # Move window
hyprctl dispatch movefocus l                   # Move focus
hyprctl dispatch togglefloating                # Toggle float
hyprctl dispatch fullscreen 1                  # Toggle fullscreen
```

3. **Configuration Commands**:
```bash
hyprctl keyword general:gaps_in 10            # Set option
hyprctl reload                                # Reload config
hyprctl setcursor Bibata-Modern-Classic 24    # Set cursor
```

4. **Batch Commands**:
```bash
hyprctl --batch "dispatch workspace 1 ; dispatch exec firefox"
```

#### 8.2 JSON Output

**Get machine-readable output:**
```bash
hyprctl -j clients
hyprctl -j monitors
hyprctl -j workspaces
```

**Example Output:**
```json
{
  "address": "0x5649c9ab0ca0",
  "at": [1920, 0],
  "size": [1920, 1080],
  "workspace": {
    "id": 1,
    "name": "1"
  },
  "floating": false,
  "monitor": 0,
  "class": "firefox",
  "title": "Mozilla Firefox",
  "pid": 12345
}
```

#### 8.3 Event System

**Subscribe to events:**
```bash
socat - UNIX-CONNECT:$XDG_RUNTIME_DIR/hypr/$HYPRLAND_INSTANCE_SIGNATURE/.socket2.sock
```

**Event Format:**
```
eventname>>data1,data2,data3
```

**Available Events:**
- `workspace>>WORKSPACENAME`
- `focusedmon>>MONNAME,WORKSPACENAME`
- `activewindow>>WINDOWCLASS,WINDOWTITLE`
- `fullscreen>>0/1`
- `monitorremoved>>MONITORNAME`
- `monitoradded>>MONITORNAME`
- `createworkspace>>WORKSPACENAME`
- `destroyworkspace>>WORKSPACENAME`
- `moveworkspace>>WORKSPACENAME,MONNAME`
- `openwindow>>ADDRESS,WORKSPACE,CLASS,TITLE`
- `closewindow>>ADDRESS`
- `movewindow>>ADDRESS,WORKSPACE`

### 9. Special Workspaces & Features

#### 9.1 Scratchpad (Special Workspace)

**Purpose:** Temporary togglable workspace for quick-access apps

**Usage:**
```ini
# Toggle special workspace visibility
bind = SUPER, S, togglespecialworkspace

# Move window to special workspace
bind = SUPER_SHIFT, S, movetoworkspace, special

# Move window from special to current
bind = SUPER_CTRL, S, movetoworkspace, current

# Named special workspaces (multiple scratchpads)
bind = SUPER, A, togglespecialworkspace, terminal
bind = SUPER_SHIFT, A, movetoworkspace, special:terminal
```

#### 9.2 Window Groups / Tabbed Containers

**Features:**
- Multiple windows in single container
- Tab-like interface
- Keyboard navigation between grouped windows

**Keybinds:**
```ini
bind = SUPER, G, togglegroup              # Create/dissolve group
bind = SUPER, TAB, changegroupactive, f   # Next window in group
bind = SUPER_SHIFT, TAB, changegroupactive, b  # Previous window
```

**Configuration:**
```ini
group {
    col.border_active = rgba(ca9ee6ff) rgba(f2d5cfff) 45deg
    col.border_inactive = rgba(b4befecc) rgba(6c7086cc) 45deg
    col.border_locked_active = rgba(ca9ee6ff) rgba(f2d5cfff) 45deg
    col.border_locked_inactive = rgba(b4befecc) rgba(6c7086cc) 45deg
    
    groupbar {
        render_titles = true
        scrolling = true
        text_color = rgba(cdd6f4ff)
        col.active = rgba(ca9ee6ff)
        col.inactive = rgba(595959aa)
    }
}
```

#### 9.3 Window Swallowing

**Purpose:** Hide terminal when GUI app launched from it

**Configuration:**
```ini
misc {
    enable_swallow = true
    swallow_regex = ^(kitty|alacritty)$
}
```

#### 9.4 Window Pinning

**Pin window to appear on all workspaces:**
```ini
bind = SUPER, P, pin
windowrulev2 = pin, title:^(Picture(-| )in(-| )[Pp]icture)$
```

### 10. Configuration System

#### 10.1 Configuration File Structure

**Main file:** `~/.config/hypr/hyprland.conf`

**Modular organization:**
```
~/.config/hypr/
├── hyprland.conf          # Main config
├── monitors.conf          # Monitor setup
├── workspaces.conf        # Workspace rules
├── windowrules.conf       # Window rules
├── keybindings.conf       # Keybindings
├── animations.conf        # Animations
├── appearance.conf        # Decorations/colors
└── autostart.conf         # Startup applications
```

**Source other files:**
```ini
source = ~/.config/hypr/monitors.conf
source = ~/.config/hypr/keybindings.conf
source = ~/.config/hypr/windowrules.conf
```

#### 10.2 Variables

```ini
$terminal = kitty
$fileManager = thunar
$menu = wofi --show drun

# Use variables
bind = SUPER, T, exec, $terminal
bind = SUPER, E, exec, $fileManager
```

#### 10.3 Hot Reload

**Reload configuration without restart:**
```bash
hyprctl reload
```

**What hot-reloads:**
- Keybindings
- Window rules
- Decorations
- Animations
- Most settings

**What requires restart:**
- Monitor configuration (sometimes)
- Some plugin changes

---

## Part 2: Waybar Features & Design

### 1. Core Status Bar Functionality

#### 1.1 Module System

**Built-in Modules:**

| Module | Purpose | Example Config |
|--------|---------|----------------|
| `clock` | Date/time display | `{"clock": {"format": "{:%H:%M}"}}` |
| `battery` | Battery status | `{"battery": {"states": {"warning": 30}}}` |
| `cpu` | CPU usage | `{"cpu": {"format": "{usage}%"}}` |
| `memory` | RAM usage | `{"memory": {"format": "{}%"}}` |
| `network` | Network status | `{"network": {"format-wifi": "{essid}"}}` |
| `pulseaudio` | Volume control | `{"pulseaudio": {"format": "{volume}%"}}` |
| `bluetooth` | BT adapter | `{"bluetooth": {"format": "{status}"}}` |
| `backlight` | Screen brightness | `{"backlight": {"format": "{percent}%"}}` |
| `idle_inhibitor` | Prevent sleep | `{"idle_inhibitor": {"format": "{icon}"}}` |
| `disk` | Disk usage | `{"disk": {"format": "{percentage_used}%"}}` |
| `temperature` | Sensors | `{"temperature": {"format": "{temperatureC}°C"}}` |
| `tray` | System tray | `{"tray": {"spacing": 10}}` |
| `mpd` | Music player | `{"mpd": {"format": "{artist} - {title}"}}` |

**Compositor-Specific Modules:**

| Module | Compositor | Purpose |
|--------|------------|---------|
| `hyprland/workspaces` | Hyprland | Workspace indicator |
| `hyprland/window` | Hyprland | Active window title |
| `hyprland/submap` | Hyprland | Current submap/mode |
| `sway/workspaces` | Sway | Workspace indicator |
| `sway/mode` | Sway | Current mode |
| `sway/window` | Sway | Active window title |

#### 1.2 Custom Modules

**Execute shell commands:**
```json
{
    "custom/weather": {
        "exec": "curl 'wttr.in/?format=3'",
        "interval": 600,
        "format": "{}",
        "tooltip": false
    },
    "custom/updates": {
        "exec": "checkupdates | wc -l",
        "interval": 3600,
        "format": " {}",
        "on-click": "kitty -e sudo pacman -Syu"
    }
}
```

### 2. Layout & Positioning

#### 2.1 Bar Configuration

**Config location:** `~/.config/waybar/config`

**Basic structure:**
```json
{
    "layer": "top",
    "position": "top",
    "height": 30,
    "spacing": 4,
    
    "modules-left": ["hyprland/workspaces", "hyprland/submap"],
    "modules-center": ["hyprland/window"],
    "modules-right": [
        "pulseaudio",
        "network",
        "cpu",
        "memory",
        "battery",
        "clock",
        "tray"
    ]
}
```

**Positioning options:**
- `"position": "top"` / `"bottom"` / `"left"` / `"right"`
- `"layer": "top"` / `"bottom"` (z-order)

#### 2.2 Per-Monitor Configuration

**Multiple bars:**
```json
[
    {
        "output": "DP-1",
        "modules-left": ["hyprland/workspaces"],
        "hyprland/workspaces": {
            "all-outputs": false
        }
    },
    {
        "output": "HDMI-A-1",
        "modules-left": ["hyprland/workspaces"],
        "hyprland/workspaces": {
            "all-outputs": false
        }
    }
]
```

### 3. Styling & Theming

#### 3.1 CSS-Based Styling

**Style location:** `~/.config/waybar/style.css`

**Basic example:**
```css
* {
    font-family: "JetBrainsMono Nerd Font";
    font-size: 13px;
}

window#waybar {
    background-color: #1e1e2e;
    color: #cdd6f4;
    border-bottom: 3px solid #89b4fa;
}

/* Workspaces */
#workspaces button {
    padding: 0 10px;
    color: #cdd6f4;
    background-color: transparent;
}

#workspaces button.active {
    background-color: #89b4fa;
    color: #1e1e2e;
}

#workspaces button:hover {
    background-color: #45475a;
}

/* Modules */
#clock,
#battery,
#cpu,
#memory,
#network,
#pulseaudio,
#tray {
    padding: 0 10px;
    margin: 0 5px;
    border-radius: 5px;
    background-color: #313244;
}

#battery.warning {
    color: #f9e2af;
}

#battery.critical {
    color: #f38ba8;
    animation: blink 1s linear infinite;
}

@keyframes blink {
    to {
        background-color: #f38ba8;
        color: #1e1e2e;
    }
}
```

#### 3.2 Popular Themes

**Catppuccin Mocha:**
```css
@define-color base #1e1e2e;
@define-color mantle #181825;
@define-color crust #11111b;
@define-color text #cdd6f4;
@define-color subtext0 #a6adc8;
@define-color blue #89b4fa;
@define-color lavender #b4befe;
@define-color red #f38ba8;
@define-color yellow #f9e2af;
@define-color green #a6e3a1;

window#waybar {
    background-color: @base;
    color: @text;
}

#workspaces button.active {
    background-color: @blue;
    color: @base;
}
```

### 4. Workspace Indicators

**Hyprland workspaces module:**
```json
{
    "hyprland/workspaces": {
        "format": "{name}: {icon}",
        "format-icons": {
            "1": "",
            "2": "",
            "3": "",
            "4": "",
            "5": "",
            "active": "",
            "default": ""
        },
        "persistent-workspaces": {
            "*": 5
        },
        "on-click": "activate",
        "on-scroll-up": "hyprctl dispatch workspace e+1",
        "on-scroll-down": "hyprctl dispatch workspace e-1"
    }
}
```

### 5. Window Title Display

```json
{
    "hyprland/window": {
        "format": "{}",
        "max-length": 50,
        "separate-outputs": true,
        "rewrite": {
            "(.*) — Mozilla Firefox": "🌎 $1",
            "(.*) - kitty": "> $1"
        }
    }
}
```

### 6. System Tray

```json
{
    "tray": {
        "icon-size": 21,
        "spacing": 10
    }
}
```

### 7. Interactive Elements

#### 7.1 Click Actions

```json
{
    "clock": {
        "format": "{:%H:%M}",
        "format-alt": "{:%Y-%m-%d}",
        "on-click-middle": "gnome-calendar"
    },
    "pulseaudio": {
        "format": "{volume}% {icon}",
        "on-click": "pavucontrol",
        "on-click-right": "pactl set-sink-mute @DEFAULT_SINK@ toggle"
    }
}
```

#### 7.2 Tooltips

```json
{
    "cpu": {
        "format": "{usage}%",
        "tooltip-format": "Load: {load}
Cores: {avg_frequency} GHz"
    },
    "custom/updates": {
        "format": " {}",
        "tooltip": true,
        "exec": "checkupdates | wc -l",
        "exec-tooltip": "checkupdates"
    }
}
```

### 8. Configuration Format

**Complete example:**
```json
{
    "layer": "top",
    "position": "top",
    "height": 30,
    "spacing": 4,
    
    "modules-left": [
        "hyprland/workspaces",
        "hyprland/submap",
        "custom/media"
    ],
    "modules-center": ["hyprland/window"],
    "modules-right": [
        "idle_inhibitor",
        "pulseaudio",
        "network",
        "cpu",
        "memory",
        "temperature",
        "backlight",
        "battery",
        "clock",
        "tray"
    ],
    
    "hyprland/workspaces": {
        "format": "{icon}",
        "format-icons": {
            "active": "",
            "default": ""
        }
    },
    
    "clock": {
        "timezone": "America/New_York",
        "format": "{:%H:%M}",
        "format-alt": "{:%Y-%m-%d}",
        "tooltip-format": "<big>{:%Y %B}</big>
<tt><small>{calendar}</small></tt>"
    },
    
    "battery": {
        "states": {
            "warning": 30,
            "critical": 15
        },
        "format": "{capacity}% {icon}",
        "format-charging": "{capacity}% ",
        "format-icons": ["", "", "", "", ""]
    }
}
```

---

## Part 3: Design Philosophy & User Experience

### 1. Hyprland Design Choices

#### 1.1 Dynamic Tiling Philosophy

**Why dynamic over manual:**
- Automatic window placement reduces cognitive load
- Consistent layouts without manual adjustment
- Faster workflow for rapidly opening/closing windows
- Still allows manual control when needed (floating, moving)

**Benefits:**
- New users can be productive immediately
- Predictable behavior
- Scales well to any screen size
- Minimizes mouse usage

#### 1.2 Performance Optimization

**Wayland-native benefits:**
- Direct rendering (no X11 overhead)
- Better input latency
- Smoother animations
- Proper multi-monitor support
- Security improvements

**Optimization strategies:**
- Hardware acceleration for compositing
- Efficient damage tracking
- Configurable animation quality
- Smart redraw scheduling
- GPU-accelerated blur/effects

#### 1.3 Animation Philosophy

**UX impact:**
- Visual feedback for actions
- Spatial awareness (where windows go)
- Professional appearance
- Smooth transitions reduce jarring changes

**Configurability:**
- Fully optional (can disable)
- Adjustable speed/curves
- Per-effect customization
- Performance scaling

#### 1.4 Extension Through IPC

**Design rationale:**
- Keep core lean and fast
- Community can build custom tools
- Scriptability enables automation
- Language-agnostic integration

### 2. Waybar Design Choices

#### 2.1 Separation of Concerns

**Functionality (JSON) vs Styling (CSS):**
- Clear responsibility boundaries
- Easy to swap themes
- Familiar web technologies
- Independent updates

**Benefits:**
- Designers can theme without coding
- Developers can add features without design
- Community sharing simplified
- Version control friendly

#### 2.2 Module Independence

**Design:**
- Each module is self-contained
- Modules communicate via IPC
- No interdependencies
- Plugin architecture ready

**Benefits:**
- Easy to add/remove modules
- Failure isolation
- Testing simplified
- Performance tuning per-module

#### 2.3 Compositor Agnostic

**Support multiple compositors:**
- Sway, Hyprland, River, etc.
- Wayland protocol based
- Fallback to generic modules

### 3. Integration Patterns

#### 3.1 Hyprland ↔ Waybar Communication

**Event-driven updates:**
```
Hyprland Event Socket
        ↓
    Waybar listens
        ↓
    Update workspace module
        ↓
    Re-render affected area
```

**IPC queries:**
```
Waybar needs data
        ↓
    hyprctl -j workspaces
        ↓
    Parse JSON response
        ↓
    Update display
```

---

## Part 4: Windows Implementation Considerations

### 1. Direct Ports (Feasible)

#### 1.1 Window Tiling Algorithm
**Status:** ✅ Fully portable

**Implementation:**
- Binary tree logic is platform-agnostic
- Use Windows API for window positioning:
  - `SetWindowPos()`
  - `MoveWindow()`
  - `GetWindowRect()`

**Rust crates:**
- `windows-rs` for Win32 API
- Custom tree structure implementation

#### 1.2 Workspace Management
**Status:** ✅ Feasible with virtual desktops

**Windows approach:**
- Use Virtual Desktop API (Windows 10+)
- Track windows per desktop
- Implement workspace switching

**API available:**
- `IVirtualDesktopManager`
- Desktop enumeration/switching

#### 1.3 Keybinding System
**Status:** ✅ Direct implementation

**Windows approach:**
- Low-level keyboard hooks: `SetWindowsHookEx()`
- Global hotkey registration: `RegisterHotKey()`
- Windows Automation (WHKD project)

**Rust implementation:**
- `winapi` crate for hooks
- `keyboard-types` for key codes

#### 1.4 IPC System
**Status:** ✅ Named pipes or TCP

**Windows implementation:**
- Named pipes: `\\.\pipe\tiling-wm`
- TCP sockets on localhost
- JSON-based protocol (same as Hyprland)

**Rust crates:**
- `tokio` for async I/O
- `serde_json` for protocol

#### 1.5 Configuration System
**Status:** ✅ Direct port

**Implementation:**
- Same INI/TOML format
- Hot reload via file watching
- `%APPDATA%` for config location

### 2. Adaptations Required

#### 2.1 Window Decorations
**Challenge:** Windows controls window frame

**Adaptation:**
- Use `WS_POPUP` style for borderless
- Custom title bar rendering (if desired)
- DWM APIs for effects:
  - `DwmSetWindowAttribute()`
  - `DwmExtendFrameIntoClientArea()`

**Compromise:**
- May need to keep Windows borders
- Or implement custom chrome

#### 2.2 Compositor/Effects
**Challenge:** DWM handles compositing

**Adaptation:**
- Can't replace DWM (closed source)
- Work with DWM, not against it
- Use DWM APIs for:
  - Blur: `DwmEnableBlurBehindWindow()`
  - Transparency: Window attributes
  - Animations: Limited to what DWM allows

**Reality:**
- Won't achieve exact Hyprland visuals
- Focus on layout, not effects

#### 2.3 Multi-Monitor
**Status:** ⚠️ Different API

**Windows approach:**
- `EnumDisplayMonitors()`
- `GetMonitorInfo()`
- Per-monitor DPI awareness

**Complications:**
- DPI scaling varies per monitor
- Must handle scale factors correctly
- `SetProcessDpiAwarenessContext()`

#### 2.4 Window Rules
**Challenge:** Different window identification

**Windows approach:**
- Process name instead of X11 class
- Window title (same)
- Window class name (Win32 class)

**Mapping:**
```rust
// Hyprland: class:^(firefox)$
// Windows: process name "firefox.exe"
match window.process_name() {
    "firefox.exe" => apply_rule(rule),
    _ => {}
}
```

### 3. Windows-Specific Enhancements

#### 3.1 PowerToys Integration
- Hook into FancyZones for zone layouts
- Complement existing Windows features
- Familiar to Windows power users

#### 3.2 Task View Integration
- Show in Windows Task View (Win+Tab)
- Integrate with Timeline
- Windows Search integration

#### 3.3 Notification Center
- Windows Action Center integration
- Toast notification support
- Focus assist integration

#### 3.4 Gaming Mode
- Detect fullscreen games
- Auto-disable tiling
- Restore on exit
- Performance priority

### 4. Technical Challenges

#### 4.1 Wayland vs Windows Compositor

| Feature | Wayland/Hyprland | Windows/DWM |
|---------|------------------|-------------|
| Compositor | Replaceable | Fixed (DWM) |
| Window control | Full | Via API |
| Effects | Custom shaders | Limited APIs |
| Input handling | Libinput | Windows Input |
| Multi-monitor | Wayland protocol | Win32 API |

#### 4.2 Permission Requirements

**Windows needs:**
- `SeDebugPrivilege` for some operations
- Run elevated for global hooks (or driver)
- UAC considerations

**Solutions:**
- Minimize privilege requirements
- Use system-allowed hooks
- Helper service for privileged ops

#### 4.3 Performance Implications

**Considerations:**
- Windows already composites via DWM
- Adding layer on top = overhead
- Must be very efficient

**Optimization:**
- Only manipulate on changes
- Cache window information
- Efficient event filtering
- Async operations

#### 4.4 Multi-DPI Handling

**Challenge:**
- Different monitors = different DPI
- Scaling factors vary
- Must calculate positions correctly

**Solution:**
```rust
// Get DPI-aware coordinates
let dpi = GetDpiForWindow(hwnd);
let scale = dpi as f32 / 96.0;
let logical_x = physical_x / scale;
```

### 5. Reference Implementation: komorebi

**Key insights from komorebi:**

1. **Architecture:**
   - Daemon process (`komorebi.exe`)
   - CLI client (`komorebic.exe`)
   - Separate hotkey daemon (WHKD)

2. **Window manipulation:**
   - `SetWindowPos()` for positioning
   - Window message hooks
   - Process monitoring

3. **Configuration:**
   - JSON-based config
   - Runtime modification via CLI
   - Workspace rules similar to Hyprland

4. **Limitations accepted:**
   - No custom compositor
   - Works with DWM limitations
   - Focus on functionality over eye-candy

**Learning:**
- Automation layer approach works
- Users accept DWM limitations
- Performance acceptable
- Feature parity achievable

---

## Feature Priority Matrix

### Must-Have (P0) - Core Functionality

| Feature | Hyprland | Windows Complexity | Dependencies |
|---------|----------|-------------------|--------------|
| Binary tree tiling (dwindle) | ✅ | Medium | None |
| Workspace management | ✅ | Medium | Virtual Desktop API |
| Window rules | ✅ | Low | Window enumeration |
| Keybindings | ✅ | Medium | Hook system |
| IPC/CLI control | ✅ | Low | Named pipes |
| Multi-monitor | ✅ | Medium | Monitor enumeration |
| Float/tile toggle | ✅ | Low | Window styles |
| Focus management | ✅ | Low | SetForegroundWindow |

### Should-Have (P1) - Important Features

| Feature | Hyprland | Windows Complexity | Dependencies |
|---------|----------|-------------------|--------------|
| Master layout | ✅ | Low | Tiling system |
| Workspace per monitor | ✅ | Medium | Virtual Desktop API |
| Window grouping | ✅ | High | Custom implementation |
| Configuration hot-reload | ✅ | Low | File watching |
| JSON output (IPC) | ✅ | Low | Serialization |
| Status bar (Waybar equiv) | ✅ | Medium | UI framework |
| Basic animations | ✅ | High | DWM APIs |
| Window borders/colors | ✅ | Medium | DWM attributes |

### Nice-to-Have (P2) - Polish Features

| Feature | Hyprland | Windows Complexity | Dependencies |
|---------|----------|-------------------|--------------|
| Scratchpad workspace | ✅ | Medium | Special workspace |
| Window swallowing | ✅ | High | Process tree |
| Bezier animations | ✅ | Very High | Custom compositor |
| Blur effects | ⚠️ | High | DWM blur API |
| Shadows | ⚠️ | Medium | DWM shadow API |
| Gradients | ⚠️ | Very High | Custom rendering |
| Submaps (modes) | ✅ | Low | Keybind system |
| Window pinning | ✅ | Low | Window flags |

### Windows-Only (P1) - Platform Features

| Feature | Description | Complexity |
|---------|-------------|------------|
| Gaming mode | Auto-disable for games | Medium |
| PowerToys integration | Work with FancyZones | High |
| Task View integration | Windows 10/11 UI | Medium |
| Notification integration | Action Center | Low |
| Win key passthrough | Start menu coexistence | Low |

---

## Windows Implementation Roadmap

### Phase 1: Core Foundation (Months 1-2)

**Goal:** Basic tiling window manager functionality

**Tasks:**
1. **Project setup**
   - Rust project structure
   - Windows API bindings (`windows-rs`)
   - Build system (Cargo)

2. **Window enumeration**
   - List all windows
   - Get window properties
   - Filter managed windows

3. **Basic tiling**
   - Binary tree structure
   - Simple two-window splits
   - Window positioning

4. **Keybinding system**
   - Global hotkey registration
   - Basic dispatchers (focus, move, close)
   - Configuration loading

**Deliverable:** Can tile 2-3 windows with keyboard control

### Phase 2: Workspace & Rules (Month 3)

**Tasks:**
1. **Virtual desktop integration**
   - Workspace creation
   - Workspace switching
   - Move windows between workspaces

2. **Window rules**
   - Process name matching
   - Title matching
   - Rule actions (float, workspace, etc.)

3. **Configuration system**
   - INI/TOML parser
   - Hot reload
   - Default config

**Deliverable:** Full workspace management + window rules

### Phase 3: IPC & Advanced Features (Month 4)

**Tasks:**
1. **IPC system**
   - Named pipe server
   - JSON protocol
   - CLI client

2. **Multi-monitor**
   - Monitor enumeration
   - Per-monitor workspaces
   - DPI awareness

3. **Layout algorithms**
   - Dwindle (complete)
   - Master layout
   - Layout switching

**Deliverable:** Full-featured tiling WM with IPC

### Phase 4: Status Bar (Month 5)

**Tasks:**
1. **Status bar framework**
   - Window creation
   - Module system
   - Configuration

2. **Core modules**
   - Workspace indicator
   - Window title
   - Clock
   - System tray

3. **Styling**
   - CSS-like theming
   - Color schemes
   - Fonts

**Deliverable:** Basic Waybar equivalent

### Phase 5: Polish & Effects (Month 6)

**Tasks:**
1. **Visual enhancements**
   - Window borders (custom or DWM)
   - Basic animations (if feasible)
   - Opacity control

2. **Special features**
   - Scratchpad workspace
   - Window groups (basic)
   - Window pinning

3. **Documentation**
   - User guide
   - Configuration examples
   - API documentation

**Deliverable:** Production-ready release v1.0

### Phase 6: Advanced Features (Ongoing)

**Tasks:**
- Advanced animations
- Plugin system
- Window swallowing
- Gaming mode
- PowerToys integration
- Community features

---

## Reference Materials

### Official Documentation

**Hyprland:**
- Official Wiki: https://wiki.hyprland.org/
- GitHub: https://github.com/hyprwm/Hyprland
- Configuration examples: https://github.com/hyprwm/Hyprland/blob/main/example/hyprland.conf

**Waybar:**
- Official Site: https://waybar.org/
- GitHub: https://github.com/Alexays/Waybar
- Wiki: https://github.com/Alexays/Waybar/wiki
- Man pages: `man 5 waybar`, `man 5 waybar-styles`

### Windows Implementation

**Komorebi (Reference Implementation):**
- GitHub: https://github.com/LGUG2Z/komorebi
- Documentation: https://lgug2z.github.io/komorebi/
- Blog: https://lgug2z.com/

**Windows APIs:**
- Desktop Window Manager: https://learn.microsoft.com/en-us/windows/win32/dwm/dwm-overview
- Virtual Desktops: https://learn.microsoft.com/en-us/windows/win32/api/_vds/
- Window Management: https://learn.microsoft.com/en-us/windows/win32/winmsg/windowing

### Rust Crates

**Windows:**
- `windows-rs`: https://github.com/microsoft/windows-rs
- `winapi`: https://github.com/retep998/winapi-rs

**Utilities:**
- `tokio`: Async runtime
- `serde`: Serialization
- `toml`: Configuration
- `notify`: File watching

### Community Resources

**Dotfiles & Configs:**
- r/unixporn: https://reddit.com/r/unixporn
- r/hyprland: https://reddit.com/r/hyprland
- Hyprland Dotfiles: https://github.com/topics/hyprland-dotfiles
- Waybar Themes: https://github.com/topics/waybar-themes

**Tutorials:**
- Hyprland setup guides on YouTube
- Waybar customization tutorials
- komorebi setup guides

### Technical Specifications

**Hyprland IPC Protocol:**
- Socket location: `$XDG_RUNTIME_DIR/hypr/$HYPRLAND_INSTANCE_SIGNATURE/.socket.sock`
- Command format: Plain text commands
- Response format: JSON (with `-j` flag)
- Event format: `eventname>>data1,data2`

**Waybar Configuration:**
- Config: `~/.config/waybar/config` (JSON)
- Style: `~/.config/waybar/style.css` (CSS3)
- Module format: See Waybar wiki for each module

**Window Tree Algorithm:**
```
Node {
    window: Option<Window>,
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
    split: Split (Horizontal/Vertical),
    geometry: Rect (x, y, width, height)
}

split_window(parent_node, new_window):
    if parent.width > parent.height:
        split = Vertical
    else:
        split = Horizontal
    
    create left_node and right_node
    attach to parent
    recalculate_geometry(parent)
```

### Code Examples

**Hyprland Window Rule:**
```ini
# Float Steam windows
windowrulev2 = float, class:^(steam|Steam)$

# Firefox to workspace 2 with opacity
windowrulev2 = workspace 2, class:^(firefox)$
windowrulev2 = opacity 0.95 0.85, class:^(firefox)$
```

**Waybar Module Config:**
```json
{
    "hyprland/workspaces": {
        "format": "{icon}",
        "format-icons": {
            "1": "",
            "2": "",
            "active": "",
            "default": ""
        },
        "on-click": "activate"
    }
}
```

**Windows Tiling (Rust pseudocode):**
```rust
use windows::Win32::UI::WindowsAndMessaging::*;

fn tile_windows(windows: &[HWND], area: RECT) {
    let tree = build_tree(windows);
    apply_geometry(tree, area);
}

fn apply_geometry(node: &Node, rect: RECT) {
    if let Some(window) = node.window {
        unsafe {
            SetWindowPos(
                window,
                None,
                rect.left,
                rect.top,
                rect.right - rect.left,
                rect.bottom - rect.top,
                SWP_NOZORDER | SWP_NOACTIVATE,
            );
        }
    } else {
        // Recursive split
        let (left_rect, right_rect) = split_rect(rect, node.split);
        if let Some(left) = &node.left {
            apply_geometry(left, left_rect);
        }
        if let Some(right) = &node.right {
            apply_geometry(right, right_rect);
        }
    }
}
```

---

## Conclusion

This research document provides a comprehensive foundation for implementing a Windows-based tiling window manager with Hyprland and Waybar feature parity. The key takeaways:

1. **Hyprland's core features are portable** to Windows with adaptation
2. **Waybar's modular design** translates well to Windows status bar
3. **komorebi proves feasibility** of the automation approach
4. **60-70% feature parity** is achievable in Phase 1-5
5. **Windows-specific enhancements** can make it better than Linux in some ways

The phased roadmap provides a realistic timeline for building a production-ready tiling window manager for Windows using Rust, with the flexibility to add features incrementally based on user feedback and priorities.

**Next Steps:**
1. Set up Rust development environment
2. Begin Phase 1 implementation
3. Create prototype with basic tiling
4. Gather early user feedback
5. Iterate and expand features

---

**Document End**
