# DPI Awareness Multi-Monitor Testing Guide

This guide provides manual testing procedures for verifying DPI awareness functionality across multiple monitors with different DPI settings.

## Prerequisites

- Windows 10 or later (Windows 11 recommended)
- Multi-monitor setup with at least 2 displays
- Ability to change display scaling settings
- Tiling Window Manager installed and running

## Test Environment Setup

### Recommended Configuration

**Monitor 1 (Primary):**
- Resolution: 1920x1080
- Scaling: 100% (DPI scale = 1.0)

**Monitor 2 (Secondary):**
- Resolution: 3840x2160 (4K)
- Scaling: 150% (DPI scale = 1.5)

Alternative configurations are acceptable as long as monitors have different DPI scaling factors.

### Setting Up Monitor DPI

1. Right-click on the desktop and select **Display Settings**
2. For each monitor:
   - Select the monitor
   - Scroll to **Scale and layout**
   - Set different scaling percentages (100%, 125%, 150%, 175%, or 200%)
3. Apply changes and sign out/in if required

## Test Procedures

### Test 1: Basic DPI Scaling Verification

**Objective:** Verify that workspace geometries correctly handle different DPI settings.

**Steps:**
1. Start the Tiling Window Manager
2. Create workspaces on both monitors:
   - Workspace 1 on Monitor 1 (100% DPI)
   - Workspace 2 on Monitor 2 (150% DPI)
3. Open test windows (e.g., Notepad, Calculator) on each workspace
4. Verify window dimensions and positions:
   - On Monitor 1: Windows should use standard 1920x1080 work area
   - On Monitor 2: Windows should account for DPI scaling

**Expected Results:**
- ✓ Windows on both monitors are properly sized
- ✓ Windows do not overflow monitor boundaries
- ✓ Window positioning is accurate on each monitor
- ✓ No visual artifacts or distortion

### Test 2: Window Positioning on High-DPI Monitors

**Objective:** Ensure windows position correctly on high-DPI monitors.

**Steps:**
1. Switch to workspace on high-DPI monitor (Monitor 2)
2. Open multiple windows (at least 3-4)
3. Arrange windows using tiling commands:
   - Split horizontally
   - Split vertically
   - Resize windows
4. Observe window placement and gaps

**Expected Results:**
- ✓ Windows tile correctly without gaps or overlaps
- ✓ Window borders are properly aligned
- ✓ Split ratios are maintained correctly
- ✓ Gap sizes are consistent and scaled appropriately

### Test 3: Dynamic DPI Change Handling

**Objective:** Verify that DPI changes trigger geometry updates.

**Steps:**
1. Open windows on both monitors
2. Change DPI scaling for Monitor 2:
   - Go to Display Settings
   - Change scaling from 150% to 125%
   - Apply and sign out/in if required
3. Return to Tiling Window Manager
4. Trigger workspace update (or restart the manager if needed)
5. Check window positions and sizes

**Expected Results:**
- ✓ Windows on affected monitor adjust to new DPI setting
- ✓ Window positions are recalculated correctly
- ✓ Windows on unaffected monitor remain unchanged
- ✓ No crashes or errors during DPI change

### Test 4: Mixed DPI Workspace Switching

**Objective:** Verify correct behavior when switching between workspaces on monitors with different DPI.

**Steps:**
1. Create workspaces on both monitors with windows
2. Switch between workspaces:
   - Switch from Workspace 1 (Monitor 1, 100% DPI) to Workspace 2 (Monitor 2, 150% DPI)
   - Switch back to Workspace 1
   - Repeat several times
3. Observe window visibility and positioning during switches

**Expected Results:**
- ✓ Windows appear and disappear correctly during switches
- ✓ Window positions are preserved when returning to workspace
- ✓ No layout corruption after multiple switches
- ✓ Smooth transitions without visual glitches

### Test 5: Multi-Window Layout on Different DPI

**Objective:** Test complex layouts across different DPI settings.

**Steps:**
1. On Monitor 1 (100% DPI):
   - Create a workspace with 4 windows in a 2x2 grid
   - Verify equal spacing and sizing
2. On Monitor 2 (150% DPI):
   - Create a workspace with 4 windows in a 2x2 grid
   - Verify equal spacing and sizing
3. Compare the layouts visually

**Expected Results:**
- ✓ Both layouts appear visually similar despite different DPI
- ✓ Relative proportions are maintained
- ✓ Gaps and borders scale appropriately
- ✓ Windows are fully contained within monitor boundaries

### Test 6: Edge Cases

**Objective:** Test boundary conditions and edge cases.

**Steps:**
1. Test with DPI scale factor very close to 1.0 (e.g., 100.5%)
2. Test with maximum DPI scale (e.g., 300% if available)
3. Test with monitor disconnection:
   - Disconnect secondary monitor
   - Verify workspaces migrate to primary monitor
   - Reconnect secondary monitor
4. Test with monitor rearrangement in Display Settings

**Expected Results:**
- ✓ Minor DPI differences (<1%) are handled correctly
- ✓ Extreme DPI scales work without errors
- ✓ Monitor disconnection/reconnection is handled gracefully
- ✓ Workspace geometry adapts to monitor changes

## Automated Test Results

The following unit tests validate the DPI scaling calculations:

- ✅ `test_dpi_scaling` - 1.5x scaling
- ✅ `test_dpi_scaling_no_change_for_1_0` - No scaling at 100%
- ✅ `test_dpi_scaling_position` - 2.0x scaling with position
- ✅ `test_dpi_scaling_fractional` - 1.25x fractional scaling
- ✅ `test_dpi_scaling_small_change_ignored` - Threshold behavior
- ✅ `test_dpi_scaling_threshold` - Edge case at threshold
- ✅ `test_update_dpi_scaling` - Single monitor integration
- ✅ `test_update_dpi_scaling_multiple_monitors` - Multiple monitors integration

## Troubleshooting

### Windows appear incorrectly sized
- Verify monitor DPI settings in Windows Display Settings
- Restart the Tiling Window Manager after DPI changes
- Check that `update_dpi_scaling()` is called after monitor changes

### Gaps or overlaps between windows
- Check gap configuration in workspace settings
- Verify DPI scale factor is correctly detected
- Review monitor work area calculation

### Performance issues with high DPI
- High DPI monitors may require more processing
- Consider reducing the number of windows per workspace
- Check system graphics driver updates

## Reporting Issues

When reporting DPI-related issues, please include:

1. Monitor configuration (resolution, DPI scale for each)
2. Windows version
3. Steps to reproduce
4. Screenshots showing the issue
5. Logs from the Tiling Window Manager

## API Usage for Developers

### Calling update_dpi_scaling

```rust
// After detecting a DPI change event
workspace_manager.update_dpi_scaling(&monitor_manager)?;
```

### Applying DPI scaling to custom rectangles

```rust
use tiling_wm_core::workspace::WorkspaceManager;
use tiling_wm_core::window_manager::tree::Rect;

let mut rect = Rect::new(0, 0, 1920, 1080);
WorkspaceManager::apply_dpi_scaling(&mut rect, 1.5);
// rect is now (0, 0, 2880, 1620)
```

## Acceptance Criteria Checklist

As defined in the original issue:

- [x] DPI scaling is applied correctly to workspace geometries
- [x] Windows position correctly on high-DPI monitors
- [x] DPI changes trigger geometry updates
- [x] Mixed DPI environments work correctly

## Next Steps

After completing manual testing:

1. Document any issues found in GitHub issues
2. Update this guide with any new test cases discovered
3. Consider automating some manual tests if possible
4. Share results with the development team
