use std::env;

pub fn install_snapshot_env() {
    env::set_var("TZ", "UTC");
    env::set_var("LC_ALL", "C");
}
