use rand::seq::SliceRandom;
use serde_json::Value;
use std::{
    cell::UnsafeCell,
    fs::File,
    io::Read,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::{JoinHandle, sleep, spawn},
    time::Duration,
};

#[derive(Debug, Clone)]
struct SharedConfig(Arc<UnsafeCell<Value>>);
unsafe impl Send for SharedConfig {}
unsafe impl Sync for SharedConfig {}

impl SharedConfig {
    fn write(&self, value: Value) {
        unsafe {
            *self.0.get() = value;
        }
    }
    fn read(&self) -> &Value {
        unsafe { &*self.0.get() }
    }
}

const CONFIG_PATH: &str = "config.json";

fn main() {
    let ready: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    #[allow(clippy::arc_with_non_send_sync)]
    let config: SharedConfig = SharedConfig(Arc::new(UnsafeCell::new(Value::Null)));

    let mut handles = Vec::<JoinHandle<()>>::new();

    // config
    let ready_clone = ready.clone();
    let config_clone = config.clone();
    handles.push(spawn(move || {
        sleep(Duration::from_millis(1000));
        let mut buf = Vec::<u8>::new();
        File::open(CONFIG_PATH)
            .unwrap()
            .read_to_end(&mut buf)
            .unwrap();
        let value: Value = serde_json::from_slice(&buf).unwrap();
        config_clone.write(value);
        ready_clone.store(true, Ordering::Release);
    }));

    // access config
    let mut fxs = vec![100, 200, 300, 400, 500];
    fxs.shuffle(&mut rand::rng());
    for (i, fx) in fxs.into_iter().enumerate() {
        let ready_clone = ready.clone();
        let config_clone = config.clone();
        handles.push(spawn(move || {
            loop {
                sleep(Duration::from_millis(fx));
                if ready_clone.load(Ordering::Acquire) {
                    println!("id:{} | fx:{} | cfg: {:?}", i, fx, config_clone.read());
                    break;
                }
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }
}
