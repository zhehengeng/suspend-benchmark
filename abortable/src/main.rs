use std::alloc::System;
use std::time::Duration;
use std::time::SystemTime;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use tokio::time::sleep;
use futures::future::{AbortHandle, Abortable};
use rand::prelude::*;
use std::thread;
use std::io::Write;
use std::collections::HashMap;
use std::thread::ThreadId;
use std::fs::OpenOptions;

static FILEPATH_WRITE: &str = "output.txt";

struct Resource {
    // counter: HashMap<(u32, u8), u64>,
    counter: HashMap<(ThreadId, u8), u64>,
    time: SystemTime,
}

impl Resource {
    fn new(num_threads: u32, num_tasks: u8) -> Self {
        Self {
            counter: HashMap::with_capacity(num_threads as usize * num_tasks as usize),
            time: SystemTime::now(),
        }
    }

    fn increment_counter(&mut self, task_id: u8) {
        if let Some(val) = self.counter.get_mut(&(thread::current().id(), task_id)) {
            *val += 1;
        } else {
            self.counter.insert((thread::current().id(), task_id), 1);
        }
    }

    fn print_to_file(&self) {
        let mut file_to_write = OpenOptions::new()
            .append(true)
            .create(true)
            .open(FILEPATH_WRITE)
            .unwrap();
        for ((thread_id, task_id), num_iter) in &self.counter {
            writeln!(file_to_write, "threadID{thread_id:?},  \
                                     taskID{task_id}, num_iter={num_iter}").unwrap();
        }
        // writeln!(file_to_write.lock().unwrap(), "Thread ID: {:?}, Task ID: {}, Counter: {}",
        //          self.thread_id, self.task_id, self.counter).unwrap();
        // writeln!(file_to_write.lock().unwrap(), "Task ID: {},",
        //          self.task_id).unwrap();
    }
}

async fn busy_loop(p_sleep: f64, sleeptime1: u64, 
                   sleeptime2: u64, task_id: u8) {
    loop {
        //resource.lock().unwrap().increment_counter(task_id);
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

// async fn read_write_file(counter: Arc<Mutex<usize>>) {
//     let mut file_to_read = OpenOptions::new()
//         .read(true)
//         .open(FILEPATH_READ)
//         .await
//         .unwrap();
//     let mut file_to_write = OpenOptions::new()
//         .append(true)
//         .create(true)
//         .open(FILEPATH_WRITE)
//         .await
//         .unwrap();
//     let mut buf = [0u8; 64 * 1024];
    
//     while let Ok(n) = file_to_read.read(&mut buf).await {
//         if n == 0 {
//             println!("not suspended");
//             break;
//         } else {
//             *(counter.lock().unwrap()) += 1;
//             file_to_write.write_all(&buf).await.unwrap();
//         }
//     }
// }

fn calculate_duration(t1: SystemTime, t2: Vec<SystemTime>) -> Vec<Duration> {
    t2.iter()
        .map(|&x| x.duration_since(t1.clone()).unwrap_or(Duration::from_micros(0))).collect()
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
    // let mut abort_registrations = Vec::with_capacity(100);
    // for _ in 0..100 {
    //     let (handle, reg) = AbortHandle::new_pair();
    //     abort_handles.push(handle);
    //     abort_registrations.push(reg);
    // }
    //let (abort_handle, abort_registration) = AbortHandle::new_pair();
    // let abort_task = tokio::spawn(async move {
    //     sleep(Duration::from_secs(5)).await;
    //     for abort_handle in abort_handles {
    //         abort_handle.abort();
    //     }
    //     println!("t1: {:?}", SystemTime::now());
    // });

    // let resource = Arc::new(Mutex::new(Resource::new(1, 100)));
    // let resource_cloned = resource.clone();
    let mut tasks = Vec::with_capacity(100);
    //let mut times = Vec::with_capacity(100);
    for task_id in 0..100 {
        let (handle, reg) = AbortHandle::new_pair();
        abort_handles.push(handle);
        let result_fut = 
            Abortable::new(busy_loop(0.5, 1, 1, task_id), 
                           reg);
        tasks.push(tokio::spawn(async move {
            match result_fut.await {
                Ok(_) => Ok(()),
                Err(_e) => Err(SystemTime::now()),
            }
        }));
    }

    // for task in tasks {
    //     task.await.unwrap();
    // }
    // abort_task.await.unwrap();
    let time = tokio::spawn(async move {
        sleep(Duration::from_secs(5)).await;
        for abort_handle in abort_handles {
            abort_handle.abort();
        }
        SystemTime::now()
    });
    let t1 = time.await.unwrap();
    // time.join();
    let mut times = Vec::with_capacity(100);
    for task in tasks {
        if let Ok(res) = task.await {
            if let Err(t) = res {
                times.push(t);
            }          
        }
        // if let Ok(_) = task.await {
        //     panic!("not finished");
        //     //println!("{e}, {:?}", SystemTime::now());
        // }
    }
    dbg!("{:?}", times.clone());
    dbg!("{}", times.len());
    let durations = calculate_duration(t1, times);
    dbg!("{}", min_duration(durations.clone()));
    dbg!("{}", max_duration(durations.clone()));
    dbg!("{}", average_durations(durations));
    //println!("{:?}", SystemTime::now());

    

    // tokio::spawn(async move {
    //     result_fut.await.unwrap();
    // });

    // sleep(Duration::from_millis(100)).await;
    // abort_handle.abort();
    // println!("counter: {}", *(counter.lock().unwrap()));
}
