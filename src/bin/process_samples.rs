use std::{io, fs, env};
use std::io::prelude::*;
use std::path::Path;

#[derive(Debug)]
struct Data {
    id: u64,
    api_duration: f64,
    latency: f64,
    gpu_duration: f64,
}

fn parse_csv(line: &str) -> Data {
    let mut inside = false;
    let mut col = line.trim().split('"').filter(|_| {
        inside = !inside;
        !inside
    });

    Data {
        id: col.next().unwrap().parse().unwrap(),
        api_duration: col.nth(2).unwrap().replace(',', "").parse().unwrap(),
        latency: col.next().unwrap().replace(',', "").parse().unwrap(),
        gpu_duration: col.nth(1).unwrap().replace(',', "").parse().unwrap(),
    }
}

fn parse_out(line: &str) -> Data {
    let mut split = line.split(',').map(|s| s.trim());
    Data {
        id: split.next().unwrap().parse().unwrap(),
        api_duration: split.next().unwrap().parse().unwrap(),
        latency: split.next().unwrap().parse().unwrap(),
        gpu_duration: split.next().unwrap().parse().unwrap(),
    }
}

fn transform_csv(input: &Path, output: &Path) {
    let mut r = io::BufReader::new(fs::File::open(input).unwrap());
    let mut w = io::BufWriter::new(fs::File::create(output).unwrap());

    let mut line = String::new();

    for _ in 0..11 {
        r.read_line(&mut line).unwrap();
        line.clear();
    }

    writeln!(w, "Transfer ID, API Duration(µs), Latency(µs), GPU Duration(µs)").unwrap();

    // Hypothesis: Sorted By GPU Start Time
    let mut prev: Option<Data> = None;
    while r.read_line(&mut line).unwrap() > 0 {
        let cur = parse_csv(&line);
        if let Some(mut p) = prev.take() {
            if p.id == cur.id {
                if p.latency < cur.latency { p.latency = cur.latency; }
                p.gpu_duration += cur.gpu_duration;
                prev = Some(p);
            } else {
                writeln!(w, "{}, {}, {}, {}", p.id, p.api_duration, p.latency, p.gpu_duration).unwrap();
                prev = Some(cur);
            }
        } else {
            prev = Some(cur);
        }

        line.clear();
    }

    if let Some(p) = prev.take() {
        writeln!(w, "{}, {}, {}, {}", p.id, p.api_duration, p.latency, p.gpu_duration).unwrap();
    }
}

struct Value {
    min: f64,
    max: f64,
    sum: f64,
}

impl Value {
    fn new() -> Value {
        Value {
            min: 1. / 0.,
            max: -1. / 0.,
            sum: 0.,
        }
    }

    fn add(&mut self, value: f64) {
        if value < self.min {
            self.min = value;
        }
        if value > self.max {
            self.max = value;
        }
        self.sum += value;
    }
}

fn process_results<W: io::Write>(directory: &Path, w: &mut W) {
    let mut api_duration = Value::new();
    let mut gpu_duration = Value::new();
    let mut n = 0f64;
    visit_dir(directory,
        &mut |path, _| if path.extension().map(|e| e == "out").unwrap_or(false) {
            let mut r = io::BufReader::new(fs::File::open(path).unwrap());
            let mut local_api_sum = 0f64;
            let mut local_gpu_sum = 0f64;
            let mut local_n = 0f64;
            let mut line = String::new();

            r.read_line(&mut line).unwrap();
            line.clear();
            while r.read_line(&mut line).unwrap() > 0 {
                let data = parse_out(&line);
                local_api_sum += data.api_duration;
                local_gpu_sum += data.gpu_duration;
                local_n += 1.;
                line.clear();
            }

            api_duration.add(local_api_sum / local_n);
            gpu_duration.add(local_gpu_sum / local_n);
            n += 1.;
        },
        &mut |_| {}
    ).unwrap();

    writeln!(w, "\"{}\", {}, {}, {}, {}, {}, {}", directory.display(),
        api_duration.min, api_duration.max, api_duration.sum / n,
        gpu_duration.min, gpu_duration.max, gpu_duration.sum / n).unwrap();
}

fn visit_dir<F, L>(dir: &Path, file_cb: &mut F, leaf_cb: &mut L) -> io::Result<()>
    where F: FnMut(&Path, fs::DirEntry), L: FnMut(&Path)
{
    if dir.is_dir() {
        let mut leaf = true;
        for entry in try!(fs::read_dir(dir)) {
            let entry = try!(entry);
            let path = entry.path();
            if path.is_dir() {
                leaf = false;
                try!(visit_dir(&path, file_cb, leaf_cb));
            } else {
                file_cb(&path, entry);
            }
        }
        if leaf {
            leaf_cb(&dir);
        }
    }
    Ok(())
}

fn main() {
    let mut args = env::args().skip(1);
    let directory = args.next().unwrap_or_else(|| String::from("samples"));

    let results = Path::new(&directory).join("results");
    let mut w = io::BufWriter::new(fs::File::create(results).unwrap());

    visit_dir(directory.as_ref(),
        &mut |path, _| if path.extension().map(|e| e == "csv").unwrap_or(false) {
            let output = path.with_extension("out");
            if !output.exists() {
                println!("{} -> {}", path.display(), output.display());
                transform_csv(path, &output);
            }
        },
        &mut |leaf_path| {
            println!("processing {}", leaf_path.display());
            process_results(leaf_path, &mut w)
        }
    ).unwrap();

    println!("done!");
}