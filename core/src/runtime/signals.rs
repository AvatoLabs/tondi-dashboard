use crate::events::Events;
use crate::runtime::Runtime;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

pub struct Signals {
    runtime: Runtime,
    iterations: AtomicU64,
}

impl Signals {
    pub fn bind(runtime: &Runtime) {
        let signals = Arc::new(Signals {
            runtime: runtime.clone(),
            iterations: AtomicU64::new(0),
        });

        ctrlc::set_handler(move || {
            let v = signals.iterations.fetch_add(1, Ordering::SeqCst);

            match v {
                0 => {
                    // post a graceful exit event to the main event loop
                    println!("^C - initiating graceful shutdown...");
                    signals.runtime.try_send(Events::Exit).unwrap_or_else(|e| {
                        println!("Error sending exit event: {:?}", e);
                    });
                    
                    // 如果主事件循环没有及时响应，强制退出
                    let signals_clone = signals.clone();
                    std::thread::spawn(move || {
                        std::thread::sleep(std::time::Duration::from_secs(8));
                        println!("^C - graceful shutdown timeout, forcing exit...");
                        signals_clone.runtime.try_send(Events::Exit).ok();
                        std::process::exit(1);
                    });
                }
                1 => {
                    // start runtime abort sequence
                    // (attempt to gracefully shutdown tondid if running)
                    // this will execute process::exit(1) after 5 seconds
                    println!("^C - forcing shutdown...");
                    crate::runtime::abort();
                }
                _ => {
                    // exit the process immediately
                    println!("^C - immediate exit");
                    std::process::exit(1);
                }
            }
        })
        .expect("Error setting signal handler");
    }
}
