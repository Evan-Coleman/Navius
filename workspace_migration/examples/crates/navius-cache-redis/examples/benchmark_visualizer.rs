use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::time::Duration;

/// A simple benchmark results visualizer
/// Run benchmarks with detailed output first:
/// `cargo bench --bench redis_benchmark -- --verbose > benchmark_results.txt`
/// Then run this visualizer:
/// `cargo run --example benchmark_visualizer -- benchmark_results.txt`

#[derive(Debug, Clone)]
struct BenchmarkResult {
    group: String,
    name: String,
    iterations: u64,
    average_time: Duration,
    min_time: Duration,
    max_time: Duration,
    throughput: Option<f64>, // operations per second
}

fn parse_time(time_str: &str) -> Duration {
    if time_str.contains("ns") {
        let ns = time_str.replace("ns", "").trim().parse::<u64>().unwrap();
        Duration::from_nanos(ns)
    } else if time_str.contains("µs") {
        let us = time_str.replace("µs", "").trim().parse::<f64>().unwrap();
        Duration::from_nanos((us * 1000.0) as u64)
    } else if time_str.contains("ms") {
        let ms = time_str.replace("ms", "").trim().parse::<f64>().unwrap();
        Duration::from_nanos((ms * 1_000_000.0) as u64)
    } else if time_str.contains("s") {
        let s = time_str.replace("s", "").trim().parse::<f64>().unwrap();
        Duration::from_nanos((s * 1_000_000_000.0) as u64)
    } else {
        panic!("Unknown time format: {}", time_str);
    }
}

fn parse_benchmark_results(file_path: &str) -> Vec<BenchmarkResult> {
    let file = File::open(file_path).expect("Failed to open benchmark results file");
    let reader = BufReader::new(file);

    let mut results = Vec::new();
    let mut current_group = String::new();
    let mut current_benchmark = String::new();

    for line in reader.lines() {
        let line = line.expect("Failed to read line");

        if line.starts_with("Benchmarking ") {
            let parts: Vec<&str> = line.split('/').collect();
            if parts.len() >= 2 {
                current_group = parts[0].replace("Benchmarking ", "").trim().to_string();
                current_benchmark = parts[1].trim().to_string();
            }
        } else if line.contains("time:") && line.contains("iterations") {
            // Parse time and iterations
            let parts: Vec<&str> = line.split(',').collect();
            let mut iterations = 0;
            let mut avg_time = Duration::from_secs(0);
            let mut min_time = Duration::from_secs(0);
            let mut max_time = Duration::from_secs(0);

            for part in parts {
                if part.contains("iterations") {
                    iterations = part.trim().replace(" iterations", "").parse().unwrap_or(0);
                } else if part.contains("time:") {
                    let time_parts: Vec<&str> = part.split(' ').collect();
                    avg_time = parse_time(time_parts.last().unwrap_or(&"0ns"));
                } else if part.contains("min") {
                    let min_parts: Vec<&str> = part.split(' ').collect();
                    min_time = parse_time(min_parts.last().unwrap_or(&"0ns"));
                } else if part.contains("max") {
                    let max_parts: Vec<&str> = part.split(' ').collect();
                    max_time = parse_time(max_parts.last().unwrap_or(&"0ns"));
                }
            }

            // Calculate throughput (ops per second)
            let throughput = if !avg_time.is_zero() {
                Some(1_000_000_000.0 / avg_time.as_nanos() as f64)
            } else {
                None
            };

            results.push(BenchmarkResult {
                group: current_group.clone(),
                name: current_benchmark.clone(),
                iterations,
                average_time: avg_time,
                min_time,
                max_time,
                throughput,
            });
        }
    }

    results
}

fn format_duration(duration: Duration) -> String {
    let ns = duration.as_nanos();

    if ns < 1_000 {
        format!("{}ns", ns)
    } else if ns < 1_000_000 {
        format!("{:.2}µs", ns as f64 / 1_000.0)
    } else if ns < 1_000_000_000 {
        format!("{:.2}ms", ns as f64 / 1_000_000.0)
    } else {
        format!("{:.2}s", ns as f64 / 1_000_000_000.0)
    }
}

fn print_bar_chart(results: &[BenchmarkResult], group_filter: Option<&str>) {
    // Filter results by group if specified
    let filtered_results: Vec<&BenchmarkResult> = if let Some(group) = group_filter {
        results.iter().filter(|r| r.group == group).collect()
    } else {
        results.iter().collect()
    };

    if filtered_results.is_empty() {
        println!("No results found for group: {:?}", group_filter);
        return;
    }

    // Sort by average time
    let mut sorted_results = filtered_results.clone();
    sorted_results.sort_by(|a, b| a.average_time.cmp(&b.average_time));

    let max_name_width = sorted_results
        .iter()
        .map(|r| r.name.len())
        .max()
        .unwrap_or(20);

    let terminal_width = 100; // Assume terminal width of 100 chars
    let max_bar_width = terminal_width - max_name_width - 20;

    let max_time = sorted_results
        .iter()
        .map(|r| r.average_time)
        .max()
        .unwrap_or(Duration::from_secs(1));

    println!("\n{}", "=".repeat(terminal_width));
    if let Some(group) = group_filter {
        println!("Benchmark Results for Group: {}", group);
    } else {
        println!("Benchmark Results Summary (Sorted by Average Time)");
    }
    println!("{}", "=".repeat(terminal_width));

    println!(
        "{:<width$} | {:<12} | {}",
        "Benchmark",
        "Time",
        "Throughput (ops/sec)",
        width = max_name_width
    );
    println!("{}", "-".repeat(terminal_width));

    for result in sorted_results {
        let bar_width = ((result.average_time.as_nanos() as f64 / max_time.as_nanos() as f64)
            * max_bar_width as f64) as usize;
        let bar = "█".repeat(bar_width);

        println!(
            "{:<width$} | {:<12} | {:<12.2} {}",
            result.name,
            format_duration(result.average_time),
            result.throughput.unwrap_or(0.0),
            bar,
            width = max_name_width
        );
    }

    println!("{}", "=".repeat(terminal_width));
}

fn print_comparison_chart(results: &[BenchmarkResult], comparison_pairs: &[(String, String)]) {
    println!("\n{}", "=".repeat(100));
    println!("Performance Comparisons");
    println!("{}", "=".repeat(100));

    let mut results_map = HashMap::new();
    for result in results {
        results_map.insert(format!("{}/{}", result.group, result.name), result);
    }

    let terminal_width = 100;

    for (benchmark1, benchmark2) in comparison_pairs {
        if let (Some(result1), Some(result2)) =
            (results_map.get(benchmark1), results_map.get(benchmark2))
        {
            let ratio =
                result1.average_time.as_nanos() as f64 / result2.average_time.as_nanos() as f64;

            println!("Comparison: {} vs {}", benchmark1, benchmark2);
            println!(
                "  {} - {}",
                result1.name,
                format_duration(result1.average_time)
            );
            println!(
                "  {} - {}",
                result2.name,
                format_duration(result2.average_time)
            );

            if ratio > 1.0 {
                println!("  Result: {} is {:.2}x faster", result2.name, ratio);
            } else {
                println!("  Result: {} is {:.2}x faster", result1.name, 1.0 / ratio);
            }

            println!("{}", "-".repeat(terminal_width));
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let file_path = if args.len() > 1 {
        &args[1]
    } else {
        "benchmark_results.txt"
    };

    let results = parse_benchmark_results(file_path);

    // Get all unique groups
    let mut groups: Vec<String> = results.iter().map(|r| r.group.clone()).collect();
    groups.sort();
    groups.dedup();

    // Print summary of all results
    print_bar_chart(&results, None);

    // Print detailed results for each group
    for group in &groups {
        print_bar_chart(&results, Some(group));
    }

    // Print specific comparisons
    let comparisons = vec![
        (
            "Pipelining/Pipeline operations 100".to_string(),
            "Pipelining/Individual operations 100".to_string(),
        ),
        (
            "Basic Operations/SET string".to_string(),
            "Basic Operations/SET with expiration".to_string(),
        ),
        (
            "Connection Pool/Concurrent operations 1".to_string(),
            "Connection Pool/Concurrent operations 50".to_string(),
        ),
    ];

    print_comparison_chart(&results, &comparisons);
}
