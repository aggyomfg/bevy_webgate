use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub(crate) struct ConnectionTracker {
    active_count: Arc<AtomicUsize>,
    total_count: Arc<AtomicUsize>,
}

impl Default for ConnectionTracker {
    fn default() -> Self {
        Self {
            active_count: Arc::new(AtomicUsize::new(0)),
            total_count: Arc::new(AtomicUsize::new(0)),
        }
    }
}

impl ConnectionTracker {
    pub fn new_connection(&self) -> ConnectionGuard {
        self.total_count.fetch_add(1, Ordering::SeqCst);
        self.active_count.fetch_add(1, Ordering::SeqCst);

        ConnectionGuard {
            active_count: Arc::clone(&self.active_count),
        }
    }

    pub fn active_connections(&self) -> usize {
        self.active_count.load(Ordering::Relaxed)
    }

    pub fn total_connections(&self) -> usize {
        self.total_count.load(Ordering::Relaxed)
    }
}

/// RAII guard that automatically decrements active connection count on drop
pub(crate) struct ConnectionGuard {
    active_count: Arc<AtomicUsize>,
}

impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        self.active_count.fetch_sub(1, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_connection_tracker_basic() {
        let tracker = ConnectionTracker::default();

        // Initially no connections
        assert_eq!(tracker.active_connections(), 0);
        assert_eq!(tracker.total_connections(), 0);

        // Create first connection
        let guard1 = tracker.new_connection();
        assert_eq!(tracker.active_connections(), 1);
        assert_eq!(tracker.total_connections(), 1);

        // Create second connection
        let guard2 = tracker.new_connection();
        assert_eq!(tracker.active_connections(), 2);
        assert_eq!(tracker.total_connections(), 2);

        // Drop first connection
        drop(guard1);
        assert_eq!(tracker.active_connections(), 1);
        assert_eq!(tracker.total_connections(), 2);

        // Drop second connection
        drop(guard2);
        assert_eq!(tracker.active_connections(), 0);
        assert_eq!(tracker.total_connections(), 2);
    }

    #[test]
    fn test_connection_tracker_clone() {
        let tracker1 = ConnectionTracker::default();
        let tracker2 = tracker1.clone();

        // Both trackers should share the same counters
        let _guard1 = tracker1.new_connection();
        assert_eq!(tracker1.active_connections(), 1);
        assert_eq!(tracker2.active_connections(), 1);

        let _guard2 = tracker2.new_connection();
        assert_eq!(tracker1.active_connections(), 2);
        assert_eq!(tracker2.active_connections(), 2);
        assert_eq!(tracker1.total_connections(), 2);
        assert_eq!(tracker2.total_connections(), 2);
    }

    #[test]
    fn test_connection_guard_raii() {
        let tracker = ConnectionTracker::default();

        {
            let _guard = tracker.new_connection();
            assert_eq!(tracker.active_connections(), 1);
        } // guard goes out of scope here

        // After guard is dropped, active count should be 0
        assert_eq!(tracker.active_connections(), 0);
        assert_eq!(tracker.total_connections(), 1);
    }

    #[test]
    fn test_multiple_guards_drop_order() {
        let tracker = ConnectionTracker::default();

        let guard1 = tracker.new_connection();
        let guard2 = tracker.new_connection();
        let guard3 = tracker.new_connection();

        assert_eq!(tracker.active_connections(), 3);
        assert_eq!(tracker.total_connections(), 3);

        // Drop in different order
        drop(guard2); // Drop middle one first
        assert_eq!(tracker.active_connections(), 2);

        drop(guard3); // Drop last one
        assert_eq!(tracker.active_connections(), 1);

        drop(guard1); // Drop first one last
        assert_eq!(tracker.active_connections(), 0);

        // Total should remain 3
        assert_eq!(tracker.total_connections(), 3);
    }
}
