//! `Core` is the one place the subsystems are assembled.
//!
//! Before it existed the desktop host hand-built three leaf objects, never held a store,
//! and re-parsed the harness registry on every invoke. These assert the properties that
//! made it worth building.

use std::path::PathBuf;

use locus_core::core::Core;

fn harnesses() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../harnesses")
}

#[test]
fn loads_the_registry_once_at_start() {
    let core = Core::load(harnesses()).expect("core loads");
    assert_eq!(core.registry().len(), 11);
}

#[test]
fn one_graph_is_shared_rather_than_rebuilt() {
    let core = Core::load(harnesses()).expect("core loads");
    let second = core.clone();

    // Two handles, one set of channels: a subscriber on either sees the other's traffic.
    let mut received = second.pty().subscribe();
    core.pty().send(b"same channel");
    assert_eq!(
        received.try_recv().expect("the shared channel delivered"),
        b"same channel".to_vec()
    );
}

#[test]
fn the_store_is_absent_until_connected() {
    // The shell starts before Postgres is reachable, and the registry surfaces do not
    // need it. `None` is a state the UI renders, not a crash.
    let core = Core::load(harnesses()).expect("core loads");
    assert!(core.store().is_none());
}

#[test]
fn the_daemon_is_owned_by_the_root_not_by_a_window() {
    let core = Core::load(harnesses()).expect("core loads");
    core.daemon().lock().expect("daemon lock").attach_window();
    assert_eq!(
        core.daemon()
            .lock()
            .expect("daemon lock")
            .attached_windows(),
        1
    );

    core.daemon().lock().expect("daemon lock").detach_window();
    // Closing the window detaches the UI and nothing else — PLAN.md §Process topology.
    assert_eq!(
        core.daemon()
            .lock()
            .expect("daemon lock")
            .attached_windows(),
        0
    );
    assert_eq!(core.registry().len(), 11);
}
