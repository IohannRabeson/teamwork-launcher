use {
    std::cell::RefCell,
    sysinfo::{ProcessRefreshKind, RefreshKind, System, SystemExt},
};

pub struct ProcessDetection {
    system: RefCell<System>,
}

impl ProcessDetection {
    pub fn is_game_detected(&self) -> bool {
        let mut system = self.system.borrow_mut();

        system.refresh_processes_specifics(ProcessRefreshKind::new());

        let mut processes = system.processes_by_exact_name("hl2.exe");

        processes.next().is_some()
    }
}

impl Default for ProcessDetection {
    fn default() -> Self {
        let this = Self {
            system: RefCell::new(System::new_with_specifics(
                RefreshKind::new().with_processes(ProcessRefreshKind::new()),
            )),
        };

        this
    }
}
