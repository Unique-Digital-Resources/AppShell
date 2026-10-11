//! Minimal binary entrypoint. Demonstrates that the App Engine runs without UI.

use app_shell::app_engine::{AppEngine, Bootstrap, Lifecycle};

fn main() {
    let mut engine: AppEngine = Bootstrap::create().expect("bootstrap failed");

    engine.initialize().expect("initialize failed");
    engine.start().expect("start failed");
    engine.run().expect("run failed");
    engine.stop().expect("stop failed");
    engine.dispose().expect("dispose failed");

    println!("AppShell lifecycle completed. Final state: {:?}", engine.state());
}