#![windows_subsystem = "windows"]
use orbital_sim::run;
fn main() {
    pollster::block_on(run());
}