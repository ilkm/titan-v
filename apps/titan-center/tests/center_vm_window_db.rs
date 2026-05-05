//! Center DB CRUD smoke tests (transport-independent).

use tempfile::tempdir;
use titan_center::app::vm_window_db::{conflicts_for, delete_by_record_id, list_all, upsert};
use titan_common::VmWindowRecord;

fn sample(record_id: &str, vm_id: u32, dir: &str) -> VmWindowRecord {
    VmWindowRecord {
        record_id: record_id.into(),
        device_id: "host-a".into(),
        host_control_addr: "127.0.0.1:7788".into(),
        host_label: "host-a".into(),
        cpu_count: 2,
        memory_mib: 1024,
        disk_mib: 2048,
        vm_directory: dir.into(),
        vm_id,
        remark: String::new(),
        created_at_unix_ms: 1,
    }
}

#[test]
fn upsert_then_list_returns_inserted_rows() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("vm.sqlite");
    upsert(&path, &sample("a", 100, "/tmp/vm/100")).unwrap();
    upsert(&path, &sample("b", 101, "/tmp/vm/101")).unwrap();
    let rows = list_all(&path).unwrap();
    assert_eq!(rows.len(), 2);
    assert!(rows.iter().any(|r| r.record_id == "a" && r.vm_id == 100));
    assert!(rows.iter().any(|r| r.record_id == "b" && r.vm_id == 101));
}

#[test]
fn conflict_check_detects_duplicate_vm_id_and_directory() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("vm.sqlite");
    upsert(&path, &sample("a", 100, "/tmp/vm/100")).unwrap();
    assert!(conflicts_for(&path, "new-record", 100, "/tmp/other").unwrap());
    assert!(conflicts_for(&path, "new-record", 999, "/tmp/vm/100").unwrap());
    assert!(!conflicts_for(&path, "new-record", 999, "/tmp/vm/999").unwrap());
}

#[test]
fn delete_removes_only_the_named_row() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("vm.sqlite");
    upsert(&path, &sample("a", 100, "/tmp/vm/100")).unwrap();
    upsert(&path, &sample("b", 101, "/tmp/vm/101")).unwrap();
    assert_eq!(delete_by_record_id(&path, "a").unwrap(), 1);
    let rows = list_all(&path).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].record_id, "b");
}
