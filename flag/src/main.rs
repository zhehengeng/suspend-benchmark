use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::*;
use std::thread::ThreadId;
use std::time::Duration;
use std::time::SystemTime;
use tokio::task::JoinHandle;
use tokio::task::futures;
use tokio::time::sleep;
use tokio::runtime::Runtime;
use rand::prelude::*;
use std::thread;
use std::sync::{Arc, Mutex};
use std::fs::OpenOptions;
use std::io::Write;
use std::collections::HashMap;
use std::process;
use chrono::Utc;
// use futures::future::join_all;

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

async fn busy_loop(flag: Arc<AtomicBool>, p_sleep: f64, sleeptime1: u64, 
                   sleeptime2: u64, task_id: u8) -> SystemTime {//, resource: Arc<Mutex<Resource>>) {
    //let mut rng = thread_rng();
    while flag.load(Relaxed) {
        //resource.lock().unwrap().increment_counter(task_id);
        let random_num = {
            let mut rng = rand::thread_rng();
            rng.gen::<f64>()
        };
        assert!(random_num >= 0.0 && random_num < 1.0);
        //let random_num: f64 = rng.gen();
        if random_num < p_sleep {
            sleep(Duration::from_millis(sleeptime1)).await;
        } else {
            sleep(Duration::from_millis(sleeptime2)).await;
        }
    }
    //resource.lock().unwrap().time = SystemTime::now();
    SystemTime::now()
}

// async fn read_write_file(flag: Arc<AtomicBool>) {
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
//     let mut counter = 0;
    
//     while let Ok(n) = file_to_read.read(&mut buf).await {
//         if !flag.load(Relaxed) {
//             println!("counter: {}. t2: {:?}", counter, SystemTime::now());
//             break;
//         }

//         if n == 0 {
//             println!("not suspended");
//             break;
//         } else {
//             counter += 1;
//             file_to_write.write_all(&buf).await.unwrap();
//         }
//     }
// }

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

// #[tokio::main(flavor = "current_thread")]
#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() {
    let flag = Arc::new(AtomicBool::new(true));
    let flag_cloned = flag.clone();
    //let resource = Arc::new(Mutex::new(Resource::new(1, 100)));
    //let resource_cloned = resource.clone();
    let mut tasks = Vec::with_capacity(100);
    let abort_task = tokio::spawn(async move {
        sleep(Duration::from_secs(5)).await;//.await;
        //let now = SystemTime::now();
        flag_cloned.store(false, Relaxed);
        //println!("t1: {:?}", SystemTime::now());
        SystemTime::now()
    });
    // tokio::spawn(async move {
    //     busy_loop(true, 0.5, 1, 10, 0).await;
    // });
    for task_id in 0..100 {
        let flag = flag.clone();
        //let resource = resource.clone();
        // tasks.push(tokio::spawn(async move {
        //     busy_loop(flag, 0.5, 1, 1, task_id).await;
        // }));
        tasks.push(tokio::spawn(
            busy_loop(flag, 0.5, 1, 1, task_id)
        ));
    }
    
    let mut outputs = Vec::with_capacity(tasks.len());
    for task in tasks {
        outputs.push(task.await.unwrap());
    }
    // let rt  = Runtime::new().unwrap();
    let t1 = abort_task.await.unwrap();
    dbg!(outputs.clone());
    println!("{}", outputs.len());
    // for task_id in 0..100 {
    //     let flag = flag.clone();
    //     let resource = resource.clone();
    //     tokio::spawn(async move {
    //         busy_loop(flag, 0.5, 1, 1, task_id, resource.clone()).await;
    //     });
    // }
    // task.await;
    // thread::spawn(move || {
    //     for task_id in 0..100 {
    //         let flag = flag.clone();
    //         let resource = resource.clone();
    //         tokio::spawn(async move {
    //             busy_loop(flag, 0.5, 1, 1, task_id, resource.clone()).await;
    //         });
    //     }
    // });
    //rt.block_on(async{wait_spawned_tasks(tasks).await});
    //join_all(tasks).await;
    // sleep(Duration::from_secs(2)).await;
    //     flag_cloned.store(false, Relaxed);
    //     println!("t1: {:?}", SystemTime::now());
    // println!("t2: {:?}", SystemTime::now());
    // resource_cloned.lock().unwrap().print_to_file();
    // println!("t2: {:?}", resource_cloned.lock().unwrap().time);
    let durations = calculate_duration(t1, outputs);
    dbg!("{}", min_duration(durations.clone()));
    dbg!("{}", max_duration(durations.clone()));
    dbg!("{}", average_durations(durations));
}
