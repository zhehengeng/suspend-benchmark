use std::time::Duration;
use std::time::SystemTime;
use tokio::time::sleep;
use futures::future::{AbortHandle, Abortable};
use rand::prelude::*;

async fn busy_loop(p_sleep: f64, sleeptime1: u64, 
                   sleeptime2: u64) {
    loop {
        let random_num = {
            let mut rng = rand::thread_rng();
            rng.gen::<f64>()
        };
        assert!(random_num >= 0.0 && random_num < 1.0);
        if random_num < p_sleep {
            sleep(Duration::from_millis(sleeptime1)).await;
        } else {
            sleep(Duration::from_millis(sleeptime2)).await;
        }
    }
}

fn calculate_duration(t1: Vec<SystemTime>, t2: Vec<SystemTime>) -> Vec<Duration> {
    t1.iter()
        .zip(t2.iter())
        .map(|(start, end)| end.duration_since(start.clone()).unwrap()).collect()
}

fn max_duration(durations: Vec<Duration>) -> Duration {
    *durations.iter()
        .max_by(|&a, &b| a.cmp(b))
        .unwrap()
}

fn min_duration(durations: Vec<Duration>) -> Duration {
    *durations.iter()
        .min_by(|&a, &b| a.cmp(b))
        .unwrap()
}

fn average_durations(durations: Vec<Duration>) -> Duration {
    let total_durations: Duration = durations.iter().sum();
    total_durations / (durations.len() as u32)
}

#[tokio::main(flavor = "current_thread")]
// #[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() {
    let mut abort_handles: Vec<AbortHandle> = Vec::with_capacity(100);
    let mut tasks = Vec::with_capacity(100);
    for _ in 0..100 {
        let (handle, reg) = AbortHandle::new_pair();
        abort_handles.push(handle);
        let result_fut = 
            Abortable::new(busy_loop(0.5, 1, 1), reg);
        tasks.push(tokio::spawn(async move {
            match result_fut.await {
                Ok(_) => Ok(()),
                Err(_e) => Err(SystemTime::now()),
            }
        }));
    }

    let mut start_times = Vec::with_capacity(100);
    sleep(Duration::from_secs(5)).await;
    for abort_handle in abort_handles {
        abort_handle.abort();
        start_times.push(SystemTime::now());
    }
    
    let mut times = Vec::with_capacity(100);
    for task in tasks {
        if let Ok(res) = task.await {
            if let Err(t) = res {
                times.push(t);
            }          
        }
    }
    
    let durations = calculate_duration(start_times, times);
    dbg!("{}", min_duration(durations.clone()));
    dbg!("{}", max_duration(durations.clone()));
    dbg!("{}", average_durations(durations));
}
