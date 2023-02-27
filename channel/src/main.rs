use std::time::Duration;
use std::time::SystemTime;
use tokio::time::sleep;
use rand::prelude::*;
use tokio::sync::broadcast;

async fn busy_loop(mut rx: broadcast::Receiver<&str>, p_sleep: f64, sleeptime1: u64, 
                   sleeptime2: u64) -> SystemTime {
    loop {
        if let Ok(_) = rx.try_recv() {
            return SystemTime::now();
        }
    
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

fn calculate_duration(t1: SystemTime, t2: Vec<SystemTime>) -> Vec<Duration> {
    t2.iter()
        .map(|&x| x.duration_since(t1.clone()).unwrap()).collect()
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
    let (tx, _) = broadcast::channel(16);
    let tx_cloned = tx.clone();
    let abort_task = tokio::spawn(async move {
        sleep(Duration::from_secs(5)).await;
        tx_cloned.send("suspend").unwrap();
        SystemTime::now()
    });
   
    let mut tasks = Vec::with_capacity(100);
    for _ in 0..100 {
        let tx = tx.clone();
        let rx = tx.subscribe();
        tasks.push(tokio::spawn(
            busy_loop(rx, 0.5, 1, 1)
        ));
    }
    
    let mut outputs = Vec::with_capacity(tasks.len());
    for task in tasks {
        outputs.push(task.await.unwrap());
    }
    
    let t1 = abort_task.await.unwrap();
    let durations = calculate_duration(t1, outputs);
    dbg!("{}", min_duration(durations.clone()));
    dbg!("{}", max_duration(durations.clone()));
    dbg!("{}", average_durations(durations));
}
