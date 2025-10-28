#[cfg(test)]
mod integration_tests {
    #[tokio::test]
    async fn test_raft_storage_persistence() {
        use control_plane::{RaftStorage, RaftLogEntry};
        use tempfile::TempDir;

        let dir = TempDir::new().unwrap();
        let storage = RaftStorage::new(dir.path()).unwrap();

        // Write entries
        let entry1 = RaftLogEntry {
            term: 1,
            index: 1,
            data: b"log_entry_1".to_vec(),
        };

        storage.append_entry(&entry1).unwrap();

        // Read back
        let retrieved = storage.get_entry(1).unwrap();
        assert_eq!(retrieved.unwrap(), entry1);
    }

    #[tokio::test]
    async fn test_raft_state_persistence() {
        use control_plane::{RaftStorage, RaftState};
        use tempfile::TempDir;

        let dir = TempDir::new().unwrap();
        let storage = RaftStorage::new(dir.path()).unwrap();

        let state = RaftState {
            current_term: 5,
            voted_for: Some(2),
            commit_index: 3,
            last_applied: 3,
        };

        storage.save_state(&state).unwrap();
        let retrieved = storage.load_state().unwrap();
        let state_ref = retrieved.unwrap();

        assert_eq!(state_ref.current_term, 5);
        assert_eq!(state_ref.voted_for, Some(2));
    }

    #[tokio::test]
    async fn test_commit_index_tracking() {
        use control_plane::RaftStorage;
        use tempfile::TempDir;

        let dir = TempDir::new().unwrap();
        let storage = RaftStorage::new(dir.path()).unwrap();

        storage.set_commit_index(42).unwrap();
        let retrieved = storage.get_commit_index().unwrap();
        assert_eq!(retrieved, 42);
    }

    #[tokio::test]
    async fn test_last_applied_tracking() {
        use control_plane::RaftStorage;
        use tempfile::TempDir;

        let dir = TempDir::new().unwrap();
        let storage = RaftStorage::new(dir.path()).unwrap();

        storage.set_last_applied(10).unwrap();
        let retrieved = storage.get_last_applied().unwrap();
        assert_eq!(retrieved, 10);
    }

    #[tokio::test]
    async fn test_log_entry_range_queries() {
        use control_plane::{RaftStorage, RaftLogEntry};
        use tempfile::TempDir;

        let dir = TempDir::new().unwrap();
        let storage = RaftStorage::new(dir.path()).unwrap();

        // Write 5 entries
        for i in 1..=5 {
            let entry = RaftLogEntry {
                term: 1,
                index: i,
                data: format!("entry_{}", i).into_bytes(),
            };
            storage.append_entry(&entry).unwrap();
        }

        // Query range [2, 4)
        let entries = storage.get_entries(2, 4).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].index, 2);
        assert_eq!(entries[1].index, 3);
    }

    #[tokio::test]
    async fn test_snapshot_and_flush() {
        use control_plane::RaftStorage;
        use tempfile::TempDir;

        let dir = TempDir::new().unwrap();
        let storage = RaftStorage::new(dir.path()).unwrap();

        // Take snapshot (flush)
        storage.snapshot(dir.path()).unwrap();
        storage.flush().unwrap();

        // Verify storage still works
        let count = storage.log_count().unwrap();
        assert_eq!(count, 0);
    }
}
