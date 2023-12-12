use std::fmt;
use std::time::Instant;

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

fn benchmark(workload: &mut dyn FnMut(), num_iters: u32, num_warmups: u32) -> TimeResult {
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

fn prefix_sum2(array_in: &[i32], array_out: &mut [i32]) {
    let mut acc = 0;
    for (&val, out) in array_in.iter().zip(array_out.iter_mut()) {
        acc += val;
        *out = acc;
    }
}

fn make_random_vector<R: rand::Rng>(length: usize, rng: &mut R) -> Vec<i32> {
    let mut v = vec![0; length];
    v.iter_mut().map(|_| rng.gen_range(0..10)).collect()
}

fn main() {
    const NUM_ELEMENTS: usize = 1e6 as usize;
    let mut rng = rand::thread_rng();
    let v: Vec<i32> = make_random_vector(NUM_ELEMENTS, &mut rng);
    let mut v1 = vec![0; v.len()];
    prefix_sum2(&v, &mut v1);

    // println!("{v:?}");
    // println!("{v1:?}");

    let mut f = || {
        prefix_sum(&v);
    };
    let res = benchmark(&mut f, 3, 0);
    let throughput = NUM_ELEMENTS as f32 / (res.mean * 1e-3 * 1e6);
    println!("{res} {throughput} Melts.s^-1");

    {
        let res = benchmark(&mut || prefix_sum2(&v, &mut v1), 3, 0);
        let throughput = NUM_ELEMENTS as f32 / (res.mean * 1e-3 * 1e6);
        println!("{res} {throughput} Melts.s^-1");
    }
}
