//! Headless daemon lifetime independent from desktop windows.

use std::collections::BTreeSet;

use uuid::Uuid;

/// `locusd` owns active runs. Desktop windows attach and detach without owning them.
#[derive(Default)]
pub struct Daemon {
    active_runs: BTreeSet<Uuid>,
    attached_windows: usize,
}

impl Daemon {
    pub fn attach_window(&mut self) {
        self.attached_windows += 1;
    }

    pub fn detach_window(&mut self) {
        self.attached_windows = self.attached_windows.saturating_sub(1);
    }

    pub fn begin_run(&mut self, run_id: Uuid) {
        self.active_runs.insert(run_id);
    }

    pub fn finish_run(&mut self, run_id: Uuid) {
        self.active_runs.remove(&run_id);
    }

    pub fn tracks(&self, run_id: Uuid) -> bool {
        self.active_runs.contains(&run_id)
    }

    pub fn attached_windows(&self) -> usize {
        self.attached_windows
    }
}

#[cfg(test)]
mod outlives_window {
    use uuid::Uuid;

    use super::Daemon;

    #[test]
    fn background_daemon_keeps_runs_when_the_last_window_closes() {
        let run_id = Uuid::new_v4();
        let mut daemon = Daemon::default();
        daemon.attach_window();
        daemon.begin_run(run_id);

        daemon.detach_window();

        assert_eq!(daemon.attached_windows(), 0);
        assert!(daemon.tracks(run_id));
    }
}
