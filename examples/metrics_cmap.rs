use std::{thread, time::Duration};

use concurrency::CmapMetrics;
use rand::Rng;

const N: usize = 2;
const M: usize = 4;

fn main() {
    let metrics = CmapMetrics::new();

    for i in 0..N {
        task_worker(i, metrics.clone());
    }

    for i in 0..M {
        request_worker(i, metrics.clone());
    }

    loop {
        thread::sleep(Duration::from_secs(1));
        println!("{}", metrics);
    }
}

fn task_worker(idx: usize, metrics: CmapMetrics) {
    thread::spawn(move || loop {
        let mut rng = rand::thread_rng();
        thread::sleep(Duration::from_millis(rng.gen_range(100..5000)));
        metrics.inc(format!("call.thread.worker.{}", idx));
    });
}

fn request_worker(idx: usize, metrics: CmapMetrics) {
    thread::spawn(move || loop {
        let mut rng = rand::thread_rng();
        thread::sleep(Duration::from_millis(rng.gen_range(100..5000)));
        metrics.inc(format!("call.request.worker.{}", idx));
    });
}
