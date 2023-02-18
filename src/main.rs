use sysinfo::{CpuExt, CpuRefreshKind, RefreshKind, System, SystemExt};

fn main() {
    let mut sys =
        System::new_with_specifics(RefreshKind::new().with_cpu(CpuRefreshKind::everything()));
    std::thread::sleep(std::time::Duration::from_millis(1100));
    sys.refresh_cpu();
    println!("{}%", sys.global_cpu_info().cpu_usage());
}
