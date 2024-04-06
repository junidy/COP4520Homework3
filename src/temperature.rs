use core::num;
use std::os::unix::process;
use std::sync::{Arc, Mutex};
use std::thread;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use std::cmp;
use rand::Rng;

// How many real-time milliseconds is one simulated minute?
const MINUTE_DELAY: u64 = 10;
const TEMP_RANGE: (f64, f64) = (-100.0, 70.0);

struct TemperatureData {
    max_temps: Vec<f64>,
    min_temps: Vec<f64>,
    // (difference, start min, end min)
    max_diff: (f64, usize, usize),
    // at index i: i min ago, (min, max) temps were recorded
    ten_min_history: Vec<(f64, f64)>
}

impl TemperatureData {
    pub fn new() -> TemperatureData {
        TemperatureData {
            max_temps: vec![TEMP_RANGE.0; 5],
            min_temps: vec![TEMP_RANGE.1; 5],
            max_diff: (0.0, 0, 0),
            ten_min_history: vec![(0.0, 0.0); 10]
        }
    }
}

pub fn run_temperature_module(num_minutes: usize, num_sensors: usize) {
    let mut sensors = Vec::new();
    let done_counter = Arc::new(AtomicUsize::new(0));
    for i in 0..num_sensors {
        let counter_clone = done_counter.clone();
        let latest_sample = Arc::new(Mutex::new(0.0));
        let latest_sample_clone = latest_sample.clone();
        let handle = thread::spawn(move || { sensor_loop(counter_clone, latest_sample_clone); });
        sensors.push((handle, latest_sample));
    }

    let mut minutes_elapsed = 0;
    let mut next_tick = Instant::now();
    let tick_time = Duration::from_millis(MINUTE_DELAY);

    let mut data = TemperatureData::new();

    while minutes_elapsed < num_minutes {
        thread::sleep(Duration::from_millis(MINUTE_DELAY));
        next_tick += tick_time;

        // unpark all threads
        for i in 0..num_sensors {
            sensors.get(i).unwrap().0.thread().unpark();
        }
        // wait until threads are finished collecting temps
        while done_counter.load(Ordering::SeqCst) < num_sensors {
            thread::sleep(Duration::from_micros(100));
        }
        // reset counter
        done_counter.store(0, Ordering::SeqCst);

        // process latest data
        let mut latest_samples = Vec::new();
        for i in 0..num_sensors {
            let lock = sensors.get(i).unwrap().1.lock().unwrap();
            latest_samples.push(*lock);
        }
        process_new_samples(&latest_samples, &mut data, minutes_elapsed);

        // print a report if an hour has elapsed
        if minutes_elapsed % 60 == 0 {
            print_report(minutes_elapsed, &data);
            data = TemperatureData::new();
        }

        minutes_elapsed += 1;
    }
}

fn process_new_samples(samples: &Vec<f64>, data: &mut TemperatureData, minutes_elapsed: usize) {
    let mut latest_min = TEMP_RANGE.1;
    let mut latest_max = TEMP_RANGE.0;
    
    for sample in samples.iter() {
        // println!("{}", sample);
        latest_min = if *sample < latest_min { *sample } else { latest_min };
        latest_max = if *sample > latest_max { *sample } else { latest_max };
    }



    for max_temp in data.max_temps.iter_mut() {
        if latest_max > *max_temp {
            // print!("MAX: {:.3?}°F\t", latest_max);
            *max_temp = latest_max;
            break;
        }
    }
    for min_temp in data.min_temps.iter_mut() {
        if latest_min < *min_temp {
            // print!("MIN: {:.3?}°F\t", latest_min);
            *min_temp = latest_min;
            break;
        }
    }

    data.ten_min_history.rotate_right(1);
    *data.ten_min_history.get_mut(0).unwrap() = (latest_min, latest_max);

    if minutes_elapsed % 60 < 10 {
        return;
    }

    let mut ten_min_min = TEMP_RANGE.1;
    let mut ten_min_max = TEMP_RANGE.0;

    for (min, max) in data.ten_min_history.iter() {
        ten_min_min = if *min < ten_min_min { *min } else { ten_min_min };
        ten_min_max = if *max > ten_min_max { *max } else { ten_min_max };
    }
    if ten_min_max - ten_min_min > data.max_diff.0 {
        // print!("NEW MAX DIFF: {:.3?}°F\t", ten_min_max - ten_min_min);
        data.max_diff = (ten_min_max - ten_min_min, minutes_elapsed % 60 - 10, minutes_elapsed % 60);
    }
}

fn print_report(minutes_elapsed: usize, data: &TemperatureData) {
    println!("\n\nHour {} Report", minutes_elapsed / 60 as usize);
    println!("\tMax temps: {:.3?}", data.max_temps);
    println!("\tMin temps: {:.3?}", data.min_temps);
    println!("\tLargest temp difference of {:.3?}°F recorded between minutes {} and {}", data.max_diff.0, data.max_diff.1, data.max_diff.2);
}

fn sensor_loop(done_counter: Arc<AtomicUsize>, sample_store: Arc<Mutex<f64>>) {
    loop {
        thread::park();
        {
            let mut lock = sample_store.lock().unwrap();
            *lock = sample_temperature();
        }
        done_counter.fetch_add(1, Ordering::SeqCst);
    }
}

fn sample_temperature() -> f64 {
    rand::thread_rng().gen_range(TEMP_RANGE.0..TEMP_RANGE.1)
}