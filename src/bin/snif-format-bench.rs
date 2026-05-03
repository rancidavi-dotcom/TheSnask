use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

use clap::Parser;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use snask::snif_fmt::format_snif;
use snask::snif_parser::{parse_snif, SnifValue};

#[derive(Parser)]
struct Args {
    #[arg(long, default_value = "bench/format_snif/out")]
    dir: String,
    #[arg(long, default_value_t = 7)]
    runs: usize,
    #[arg(
        long,
        default_value = "",
        help = "Internal: run only one format and print a JSON row to stdout."
    )]
    one: String,
}

#[derive(Serialize, Deserialize)]
struct Row {
    format: String,
    input_bytes: usize,
    canon_bytes: usize,
    parse_ms: f64,
    canon_ms: f64,
    total_ms: f64,
    peak_rss_kb: i64,
    sha256: String,
}

#[derive(Serialize, Deserialize)]
struct ConfigDoc {
    package: serde_json::Value,
    build: serde_json::Value,
    env: serde_json::Value,
    features: serde_json::Value,
    users: serde_json::Value,
    services: serde_json::Value,
    metadata: serde_json::Value,
}

fn now_ms(d: std::time::Duration) -> f64 {
    d.as_nanos() as f64 / 1_000_000.0
}

fn ru_maxrss_kb() -> i64 {
    unsafe {
        let mut usage: libc::rusage = std::mem::zeroed();
        if libc::getrusage(libc::RUSAGE_SELF, &mut usage) != 0 {
            return -1;
        }
        // Linux: ru_maxrss is in kilobytes.
        usage.ru_maxrss
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    let out = h.finalize();
    out.iter().map(|b| format!("{:02x}", b)).collect()
}

fn median(mut xs: Vec<f64>) -> f64 {
    xs.sort_by(|a, b| a.total_cmp(b));
    if xs.is_empty() {
        return 0.0;
    }
    xs[xs.len() / 2]
}

fn canon_json(v: &serde_json::Value) -> Vec<u8> {
    // Ensure stable key ordering by converting objects to BTreeMap recursively.
    fn to_ordered(v: &serde_json::Value) -> serde_json::Value {
        match v {
            serde_json::Value::Object(o) => {
                let mut m: BTreeMap<String, serde_json::Value> = BTreeMap::new();
                for (k, vv) in o.iter() {
                    m.insert(k.clone(), to_ordered(vv));
                }
                serde_json::Value::Object(m.into_iter().collect())
            }
            serde_json::Value::Array(a) => {
                serde_json::Value::Array(a.iter().map(to_ordered).collect())
            }
            _ => v.clone(),
        }
    }
    let ordered = to_ordered(v);
    serde_json::to_vec(&ordered).unwrap_or_default()
}

fn canon_toml(doc: &impl Serialize) -> Vec<u8> {
    // TOML serializer is stable for a fixed struct layout; we accept that as "canon".
    toml::to_string(doc).unwrap_or_default().into_bytes()
}

fn canon_yaml(doc: &impl Serialize) -> Vec<u8> {
    // YAML emitter is not universally canonical, but serde_yaml output is stable enough for same input in this harness.
    serde_yaml::to_string(doc).unwrap_or_default().into_bytes()
}

fn canon_cbor(doc: &impl Serialize) -> Vec<u8> {
    serde_cbor::to_vec(doc).unwrap_or_default()
}

fn canon_msgpack(doc: &impl Serialize) -> Vec<u8> {
    rmp_serde::to_vec_named(doc).unwrap_or_default()
}

fn run_one<F>(runs: usize, mut f: F) -> (f64, f64, usize, String, i64)
where
    F: FnMut() -> (f64, f64, Vec<u8>),
{
    let mut parse_ms = Vec::with_capacity(runs);
    let mut canon_ms = Vec::with_capacity(runs);
    let mut canon_bytes = 0usize;
    let mut sha = String::new();
    let mut peak = -1;
    for _ in 0..runs {
        let (p, c, out) = f();
        parse_ms.push(p);
        canon_ms.push(c);
        canon_bytes = out.len();
        sha = sha256_hex(&out);
        peak = peak.max(ru_maxrss_kb());
    }
    (median(parse_ms), median(canon_ms), canon_bytes, sha, peak)
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let dir = PathBuf::from(args.dir);

    // Child mode: run a single format and print JSON row to stdout.
    if !args.one.is_empty() {
        let row = run_single(&dir, args.runs, &args.one)?;
        println!("{}", serde_json::to_string(&row)?);
        return Ok(());
    }

    // Master mode: spawn a fresh process per format so peak RSS is per-format
    // (ru_maxrss is process-lifetime max).
    let mut rows: Vec<Row> = Vec::new();

    let formats = ["json", "toml", "yaml", "cbor", "msgpack", "snif"];
    for fmt in formats {
        let outp = Command::new(std::env::current_exe()?)
            .arg("--dir")
            .arg(&dir)
            .arg("--runs")
            .arg(args.runs.to_string())
            .arg("--one")
            .arg(fmt)
            .output()?;
        if !outp.status.success() {
            return Err(anyhow::anyhow!(
                "child bench failed for {}: {}",
                fmt,
                String::from_utf8_lossy(&outp.stderr)
            ));
        }
        let line = String::from_utf8_lossy(&outp.stdout);
        let row: Row = serde_json::from_str(line.trim())?;
        rows.push(row);
    }

    // Write machine-readable + markdown report
    let json_out = serde_json::to_vec_pretty(&rows)?;
    fs::write(dir.join("results.json"), json_out)?;

    let mut md = String::new();
    md.push_str("# SNIF vs JSON/TOML/YAML/CBOR/MsgPack — parse + canon (config)\n\n");
    md.push_str("Source of truth:\n- `bench/format_snif/out/results.json`\n\n");
    md.push_str("| Format | Input (MB) | Canon (MB) | Parse p50 (ms) | Canon p50 (ms) | Total (ms) | Peak RSS (MiB) |\n");
    md.push_str("| --- | ---:| ---:| ---:| ---:| ---:| ---:|\n");
    for r in &rows {
        let in_mb = r.input_bytes as f64 / (1024.0 * 1024.0);
        let out_mb = r.canon_bytes as f64 / (1024.0 * 1024.0);
        let peak_mib = (r.peak_rss_kb as f64) / 1024.0;
        md.push_str(&format!(
            "| `{}` | {:.2} | {:.2} | {:.2} | {:.2} | {:.2} | {:.1} |\n",
            r.format, in_mb, out_mb, r.parse_ms, r.canon_ms, r.total_ms, peak_mib
        ));
    }
    md.push_str("\nNotes:\n- Peak RSS is best-effort (`getrusage`).\n- Canon output hashes are recorded in `results.json`.\n");
    fs::write(dir.join("report.md"), md)?;

    Ok(())
}

fn run_single(dir: &PathBuf, runs: usize, format: &str) -> anyhow::Result<Row> {
    match format {
        "json" => {
            let json_src = fs::read(dir.join("config.json"))?;
            let input_bytes = json_src.len();
            let (p50, c50, canon_bytes, sha, peak) = run_one(runs, || {
                let t0 = Instant::now();
                let v: serde_json::Value = serde_json::from_slice(&json_src).unwrap();
                let parse = now_ms(t0.elapsed());
                let t1 = Instant::now();
                let out = canon_json(&v);
                let canon = now_ms(t1.elapsed());
                (parse, canon, out)
            });
            Ok(Row {
                format: "json".into(),
                input_bytes,
                canon_bytes,
                parse_ms: p50,
                canon_ms: c50,
                total_ms: p50 + c50,
                peak_rss_kb: peak,
                sha256: sha,
            })
        }
        "toml" => {
            let toml_src = fs::read_to_string(dir.join("config.toml"))?;
            let input_bytes = toml_src.as_bytes().len();
            let (p50, c50, canon_bytes, sha, peak) = run_one(runs, || {
                let t0 = Instant::now();
                let doc: ConfigDoc = toml::from_str(&toml_src).unwrap();
                let parse = now_ms(t0.elapsed());
                let t1 = Instant::now();
                let out = canon_toml(&doc);
                let canon = now_ms(t1.elapsed());
                (parse, canon, out)
            });
            Ok(Row {
                format: "toml".into(),
                input_bytes,
                canon_bytes,
                parse_ms: p50,
                canon_ms: c50,
                total_ms: p50 + c50,
                peak_rss_kb: peak,
                sha256: sha,
            })
        }
        "yaml" => {
            let yaml_src = fs::read_to_string(dir.join("config.yaml"))?;
            let input_bytes = yaml_src.as_bytes().len();
            let (p50, c50, canon_bytes, sha, peak) = run_one(runs, || {
                let t0 = Instant::now();
                let doc: ConfigDoc = serde_yaml::from_str(&yaml_src).unwrap();
                let parse = now_ms(t0.elapsed());
                let t1 = Instant::now();
                let out = canon_yaml(&doc);
                let canon = now_ms(t1.elapsed());
                (parse, canon, out)
            });
            Ok(Row {
                format: "yaml".into(),
                input_bytes,
                canon_bytes,
                parse_ms: p50,
                canon_ms: c50,
                total_ms: p50 + c50,
                peak_rss_kb: peak,
                sha256: sha,
            })
        }
        "cbor" => {
            let cbor_src = fs::read(dir.join("config.cbor"))?;
            let input_bytes = cbor_src.len();
            let (p50, c50, canon_bytes, sha, peak) = run_one(runs, || {
                let t0 = Instant::now();
                let doc: ConfigDoc = serde_cbor::from_slice(&cbor_src).unwrap();
                let parse = now_ms(t0.elapsed());
                let t1 = Instant::now();
                let out = canon_cbor(&doc);
                let canon = now_ms(t1.elapsed());
                (parse, canon, out)
            });
            Ok(Row {
                format: "cbor".into(),
                input_bytes,
                canon_bytes,
                parse_ms: p50,
                canon_ms: c50,
                total_ms: p50 + c50,
                peak_rss_kb: peak,
                sha256: sha,
            })
        }
        "msgpack" => {
            let msgpack_src = fs::read(dir.join("config.msgpack"))?;
            let input_bytes = msgpack_src.len();
            let (p50, c50, canon_bytes, sha, peak) = run_one(runs, || {
                let t0 = Instant::now();
                let doc: ConfigDoc = rmp_serde::from_slice(&msgpack_src).unwrap();
                let parse = now_ms(t0.elapsed());
                let t1 = Instant::now();
                let out = canon_msgpack(&doc);
                let canon = now_ms(t1.elapsed());
                (parse, canon, out)
            });
            Ok(Row {
                format: "msgpack".into(),
                input_bytes,
                canon_bytes,
                parse_ms: p50,
                canon_ms: c50,
                total_ms: p50 + c50,
                peak_rss_kb: peak,
                sha256: sha,
            })
        }
        "snif" => {
            let snif_src = fs::read_to_string(dir.join("config.snif"))?;
            let input_bytes = snif_src.as_bytes().len();
            let (p50, c50, canon_bytes, sha, peak) = run_one(runs, || {
                let t0 = Instant::now();
                let v: SnifValue = parse_snif(&snif_src).unwrap();
                let parse = now_ms(t0.elapsed());
                let t1 = Instant::now();
                let out = format_snif(&v).into_bytes();
                let canon = now_ms(t1.elapsed());
                (parse, canon, out)
            });
            Ok(Row {
                format: "snif".into(),
                input_bytes,
                canon_bytes,
                parse_ms: p50,
                canon_ms: c50,
                total_ms: p50 + c50,
                peak_rss_kb: peak,
                sha256: sha,
            })
        }
        _ => Err(anyhow::anyhow!("unknown format: {}", format)),
    }
}
