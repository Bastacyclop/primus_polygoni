use std::io;
use std::io::prelude::*;
use std::fs;
use std::env;

#[derive(Debug)]
struct Data {
    id: u64,
    api_duration: f64,
    latency: f64,
    gpu_duration: f64,
}

fn parse(line: &str) -> Data {
    let mut inside = false;
    let mut col = line.trim().split('"').filter(|_| { 
        inside = !inside;
        !inside 
    });
    let id = col.next().unwrap().parse().unwrap();
    let api_duration = col.nth(2).unwrap().replace(',', "").parse().unwrap();
    let latency = col.next().unwrap().replace(',', "").parse().unwrap();
    let gpu_duration = col.nth(1).unwrap().replace(',', "").parse().unwrap();

    Data {
        id: id,
        api_duration: api_duration,
        latency: latency,
        gpu_duration: gpu_duration,
    }
}

fn main() {
    let mut args = env::args().skip(1);
    let input = args.next().expect("input file");
    let output = args.next().expect("output file");

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
        let cur = parse(&line);
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