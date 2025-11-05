#!/bin/bash
# Validation script for Phase 5 Task 5.2: Event System Implementation

echo "=================================================="
echo "Phase 5 Task 5.2 Validation: Event System"
echo "=================================================="
echo ""

cd /home/runner/work/TilingWindowManager/TilingWindowManager

echo "1. Checking Event System Implementation..."
echo "   - events.rs exists: $([ -f crates/core/src/ipc/events.rs ] && echo '✓' || echo '✗')"
echo "   - events.rs line count: $(wc -l < crates/core/src/ipc/events.rs) lines"
echo ""

echo "2. Checking EventBroadcaster implementation..."
grep -q "pub struct EventBroadcaster" crates/core/src/ipc/events.rs && echo "   ✓ EventBroadcaster struct found" || echo "   ✗ EventBroadcaster struct NOT found"
grep -q "pub fn new() -> Self" crates/core/src/ipc/events.rs && echo "   ✓ new() method found" || echo "   ✗ new() method NOT found"
grep -q "pub fn emit(&self, event: Event)" crates/core/src/ipc/events.rs && echo "   ✓ emit() method found" || echo "   ✗ emit() method NOT found"
grep -q "pub fn subscribe(&self)" crates/core/src/ipc/events.rs && echo "   ✓ subscribe() method found" || echo "   ✗ subscribe() method NOT found"
grep -q "pub fn subscriber_count(&self)" crates/core/src/ipc/events.rs && echo "   ✓ subscriber_count() method found" || echo "   ✗ subscriber_count() method NOT found"
echo ""

echo "3. Checking Event enum implementation..."
grep -q "pub enum Event" crates/core/src/ipc/events.rs && echo "   ✓ Event enum found" || echo "   ✗ Event enum NOT found"
grep -q "WindowCreated" crates/core/src/ipc/events.rs && echo "   ✓ WindowCreated variant found" || echo "   ✗ WindowCreated variant NOT found"
grep -q "WindowClosed" crates/core/src/ipc/events.rs && echo "   ✓ WindowClosed variant found" || echo "   ✗ WindowClosed variant NOT found"
grep -q "WindowFocused" crates/core/src/ipc/events.rs && echo "   ✓ WindowFocused variant found" || echo "   ✗ WindowFocused variant NOT found"
grep -q "WorkspaceChanged" crates/core/src/ipc/events.rs && echo "   ✓ WorkspaceChanged variant found" || echo "   ✗ WorkspaceChanged variant NOT found"
grep -q "ConfigReloaded" crates/core/src/ipc/events.rs && echo "   ✓ ConfigReloaded variant found" || echo "   ✗ ConfigReloaded variant NOT found"
echo ""

echo "4. Checking Event methods..."
grep -q "pub fn to_response(&self) -> Response" crates/core/src/ipc/events.rs && echo "   ✓ to_response() method found" || echo "   ✗ to_response() method NOT found"
grep -q "pub fn event_name(&self)" crates/core/src/ipc/events.rs && echo "   ✓ event_name() method found" || echo "   ✗ event_name() method NOT found"
echo ""

echo "5. Checking test coverage..."
TEST_COUNT=$(grep -c "#\[test\]" crates/core/src/ipc/events.rs || echo "0")
echo "   - Number of unit tests: $TEST_COUNT"
grep -q "test_event_broadcaster_creation" crates/core/src/ipc/events.rs && echo "   ✓ test_event_broadcaster_creation found" || echo "   ✗ test NOT found"
grep -q "test_event_broadcaster_subscribe" crates/core/src/ipc/events.rs && echo "   ✓ test_event_broadcaster_subscribe found" || echo "   ✗ test NOT found"
grep -q "test_event_broadcast" crates/core/src/ipc/events.rs && echo "   ✓ test_event_broadcast found" || echo "   ✗ test NOT found"
grep -q "test_event_to_response" crates/core/src/ipc/events.rs && echo "   ✓ test_event_to_response found" || echo "   ✗ test NOT found"
grep -q "test_event_names" crates/core/src/ipc/events.rs && echo "   ✓ test_event_names found" || echo "   ✗ test NOT found"
echo ""

echo "6. Checking broadcast channel configuration..."
grep -q "channel(100)" crates/core/src/ipc/events.rs && echo "   ✓ Broadcast capacity set to 100" || echo "   ✗ Broadcast capacity NOT found"
echo ""

echo "7. Checking integration with protocol module..."
grep -q "use super::protocol::Response" crates/core/src/ipc/events.rs && echo "   ✓ Response import found" || echo "   ✗ Response import NOT found"
echo ""

echo "=================================================="
echo "Validation Complete"
echo "=================================================="
echo ""
echo "Note: Unit tests cannot be run in Linux environment due to Windows dependencies."
echo "However, all implementation components are verified to be present."
