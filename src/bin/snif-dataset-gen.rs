use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use clap::Parser;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use serde::{Serialize, Deserialize};

use snask::snif_fmt::format_snif;
use snask::snif_parser::SnifValue;

#[derive(Parser)]
struct Args {
    #[arg(long, default_value_t = 100)]
    target_mb: usize,
    #[arg(long, default_value = "bench/format_snif/out")]
    out_dir: String,
    #[arg(long, default_value_t = 0)]
    seed: u64,
}

#[derive(Serialize, Deserialize)]
struct ConfigDoc {
    package: Package,
    build: Build,
    env: BTreeMap<String, String>,
    features: BTreeMap<String, bool>,
    users: Vec<User>,
    services: Vec<Service>,
    metadata: BTreeMap<String, String>,
}

#[derive(Serialize, Deserialize)]
struct Package {
    name: String,
    version: String,
    entry: String,
    description: String,
}

#[derive(Serialize, Deserialize)]
struct Build {
    profile: String,
    opt_level: u8,
    strip: bool,
    lto: String,
}

#[derive(Serialize, Deserialize)]
struct User {
    id: u64,
    name: String,
    role: String,
    enabled: bool,
    tags: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct Service {
    name: String,
    url: String,
    timeout_ms: u64,
    retries: u8,
    headers: BTreeMap<String, String>,
}

fn rand_ascii(rng: &mut StdRng, len: usize) -> String {
    const A: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_-";
    let mut s = String::with_capacity(len);
    for _ in 0..len {
        s.push(A[rng.gen_range(0..A.len())] as char);
    }
    s
}

fn make_doc(rng: &mut StdRng, users: usize, services: usize) -> ConfigDoc {
    let mut env = BTreeMap::new();
    for i in 0..64 {
        env.insert(format!("KEY_{}", i), rand_ascii(rng, 32));
    }

    let mut features = BTreeMap::new();
    for i in 0..64 {
        features.insert(format!("feature_{}", i), rng.gen_bool(0.5));
    }

    let mut metadata = BTreeMap::new();
    for i in 0..128 {
        metadata.insert(format!("meta_{}", i), rand_ascii(rng, 48));
    }

    let mut users_vec = Vec::with_capacity(users);
    for i in 0..users {
        users_vec.push(User {
            id: i as u64,
            name: format!("user_{}", rand_ascii(rng, 12)),
            role: if i % 5 == 0 { "admin".into() } else { "member".into() },
            enabled: rng.gen_bool(0.9),
            tags: (0..6).map(|_| rand_ascii(rng, 8)).collect(),
        });
    }

    let mut services_vec = Vec::with_capacity(services);
    for i in 0..services {
        let mut headers = BTreeMap::new();
        for h in 0..12 {
            headers.insert(format!("X-H{}", h), rand_ascii(rng, 16));
        }
        services_vec.push(Service {
            name: format!("svc_{}_{}", i, rand_ascii(rng, 10)),
            url: format!("https://{}.example.com/api/{}", rand_ascii(rng, 12), rand_ascii(rng, 8)),
            timeout_ms: rng.gen_range(500..8000),
            retries: rng.gen_range(0..8) as u8,
            headers,
        });
    }

    ConfigDoc {
        package: Package {
            name: format!("app_{}", rand_ascii(rng, 12)),
            version: "1.2.3".into(),
            entry: "main.snask".into(),
            description: rand_ascii(rng, 80),
        },
        build: Build { profile: "release-size".into(), opt_level: 2, strip: true, lto: "thin".into() },
        env,
        features,
        users: users_vec,
        services: services_vec,
        metadata,
    }
}

fn snif_from_json_value(v: &serde_json::Value) -> SnifValue {
    match v {
        serde_json::Value::Null => SnifValue::Null,
        serde_json::Value::Bool(b) => SnifValue::Bool(*b),
        serde_json::Value::Number(n) => SnifValue::Number(n.as_f64().unwrap_or(0.0)),
        serde_json::Value::String(s) => SnifValue::String(s.clone()),
        serde_json::Value::Array(a) => SnifValue::Array(a.iter().map(snif_from_json_value).collect()),
        serde_json::Value::Object(o) => {
            let mut m = BTreeMap::new();
            for (k, vv) in o.iter() {
                m.insert(k.clone(), snif_from_json_value(vv));
            }
            SnifValue::Object(m)
        }
    }
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let out_dir = PathBuf::from(args.out_dir);
    fs::create_dir_all(&out_dir)?;

    // Build a document and scale it by increasing list sizes until JSON hits target.
    let mut rng = StdRng::seed_from_u64(args.seed);
    let target_bytes = args.target_mb * 1024 * 1024;

    let mut users = 5_000usize;
    let mut services = 1_000usize;
    let mut doc = make_doc(&mut rng, users, services);
    let mut json = serde_json::to_string(&doc)?;

    while json.len() < target_bytes {
        users = (users as f64 * 1.15) as usize + 1;
        services = (services as f64 * 1.10) as usize + 1;
        doc = make_doc(&mut rng, users, services);
        json = serde_json::to_string(&doc)?;
    }

    let json_path = out_dir.join("config.json");
    fs::write(&json_path, &json)?;

    let toml_str = toml::to_string(&doc)?;
    fs::write(out_dir.join("config.toml"), toml_str)?;

    let yaml_str = serde_yaml::to_string(&doc)?;
    fs::write(out_dir.join("config.yaml"), yaml_str)?;

    let cbor_bytes = serde_cbor::to_vec(&doc)?;
    fs::write(out_dir.join("config.cbor"), cbor_bytes)?;

    let msgpack_bytes = rmp_serde::to_vec_named(&doc)?;
    fs::write(out_dir.join("config.msgpack"), msgpack_bytes)?;

    // SNIF: convert via serde_json Value -> SnifValue to keep structure aligned.
    let json_value: serde_json::Value = serde_json::from_str(&json)?;
    let snif_value = snif_from_json_value(&json_value);
    let snif_text = format_snif(&snif_value);
    fs::write(out_dir.join("config.snif"), snif_text)?;

    // Sizes summary
    let mut summary = String::new();
    for (name, p) in [
        ("json", json_path),
        ("toml", out_dir.join("config.toml")),
        ("yaml", out_dir.join("config.yaml")),
        ("cbor", out_dir.join("config.cbor")),
        ("msgpack", out_dir.join("config.msgpack")),
        ("snif", out_dir.join("config.snif")),
    ] {
        let sz = fs::metadata(&p)?.len();
        summary.push_str(&format!("{}: {} bytes\n", name, sz));
    }
    fs::write(out_dir.join("sizes.txt"), summary)?;

    Ok(())
}

