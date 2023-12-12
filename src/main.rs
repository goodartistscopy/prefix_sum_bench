use std::fmt;
use std::time::Instant;
use rayon::prelude::*;

#[derive(Debug)]
struct TimeResult {
    mean: f32,
    stddev: f32,
}

impl fmt::Display for TimeResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}ms (±{})", self.mean, self.stddev)
    }
}

fn benchmark<F>(mut workload: F, num_iters: u32, num_warmups: u32) -> TimeResult 
    where F: FnMut()
{
    for _ in 0..num_warmups {
        workload();
    }
    let mut mean = 0.0;
    let mut stddev = 0.0;
    for _ in 0..num_iters {
        let start = Instant::now();
        workload();
        let duration = start.elapsed().as_secs_f32() * 1e3;
        mean += duration;
        stddev += duration * duration;
    }
    mean /= num_iters as f32;
    stddev = (stddev / (num_iters as f32) - mean * mean).sqrt();
    TimeResult { mean, stddev }
}

fn prefix_sum(array: &[i32]) -> Vec<i32> {
    let mut sums = Vec::new();
    for (i, &val) in array.iter().enumerate() {
        if i == 0 {
            sums.push(val);
        } else {
            sums.push(val + sums[i - 1]);
        }
    }
    sums
}

fn prefix_sum2(array_in: &[i32], array_out: &mut [i32]) -> i32 {
    let mut acc = 0;
    for (val, out) in array_in.iter().zip(array_out.iter_mut()) {
        acc += val;
        *out = acc;
    }
    return acc;
}

fn prefix_sum_par(array_in: &[i32], array_out: &mut [i32], max_num_threads: usize) {
    if max_num_threads < 2 || array_in.len() < 2 || max_num_threads > 2 * array_in.len() {
        prefix_sum2(array_in, array_out);
        return;
    }
    let chunk_size = array_in.len() / max_num_threads;
    array_in
        .par_chunks(chunk_size)
        .zip(array_out.par_chunks_mut(chunk_size))
        .for_each(|(chunk_a, chunk_b)| {
            prefix_sum2(chunk_a, chunk_b);
        });

    let partial_sums: Vec<i32> = array_out.iter().skip(chunk_size-1).step_by(chunk_size).cloned().collect();
    let prefix_partial_sums = prefix_sum(&partial_sums);
    
    array_out
        .par_chunks_mut(chunk_size)
        .skip(1)
        .enumerate()
        .for_each(|(i, chunk)| {
            chunk.iter_mut().for_each(|x| *x += prefix_partial_sums[i]); 
        });

}

fn make_random_vector<R: rand::Rng>(length: usize, rng: &mut R) -> Vec<i32> {
    let mut v = vec![0; length];
    v.iter_mut().map(|_| rng.gen_range(0..10)).collect()
}

fn main() {
    const NUM_ELEMENTS: usize = 1e6 as usize;
    const N_RUNS: u32 = 100u32;
    const N_WARMUPS: u32 = 10u32;

    let mut rng = rand::thread_rng();
    let num_threads = std::thread::available_parallelism().unwrap().get();
    let v: Vec<i32> = make_random_vector(NUM_ELEMENTS, &mut rng);
    let mut v1 = vec![0; v.len()];
    
    //println!("{v:?}");
    prefix_sum2(&v, &mut v1);
    //println!("{v1:?}");
    prefix_sum_par(&v, &mut v1, num_threads);
    //println!("{v1:?}");

    {
        let res = benchmark(&mut || { prefix_sum2(&v, &mut v1); }, N_RUNS, N_WARMUPS);
        let throughput = NUM_ELEMENTS as f32 / (res.mean * 1e-3 * 1e6);
        println!("Sequential: {res} {throughput} Melts.s^-1");
    }

    {
        let res = benchmark(&mut || { prefix_sum_par(&v, &mut v1, num_threads); }, N_RUNS, N_WARMUPS);
        let throughput = NUM_ELEMENTS as f32 / (res.mean * 1e-3 * 1e6);
        println!("Parallel: {res} {throughput} Melts.s^-1");
    }
}
