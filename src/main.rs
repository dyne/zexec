#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use std::{
    collections::HashMap,
    ffi::CString,
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result};
use clap::Clap;
use json;
use rand::{rngs::OsRng, RngCore};
use rocket::{config::Environment, response::NamedFile, Config, Data, State};
use zenroom::zencode_exec;

struct ZexecConfig {
    logs_dir: String,
}

const BUFFER_LIMIT: u64 = 2 * 1024 * 1024; // 2 Mb

#[derive(Clap)]
#[clap(version = "0.1", author = "Danilo Spinella <oss@danyspin97.org>")]
struct Opts {
    #[clap(short, long, default_value = "127.0.0.1")]
    address: String,
    #[clap(short, long, default_value = "/etc/zexec")]
    contracts_dir: String,
    #[clap(short, long, default_value = "/var/lib/zexec/logs")]
    logs_dir: String,
    #[clap(short, long, default_value = "9856")]
    port: u16,
}

fn get_random_name() -> String {
    let mut data = [0u8; 16];
    OsRng.fill_bytes(&mut data);
    data.iter().map(|byte| format!("{:x}", byte)).collect()
}

fn get_contracts(contract_dir: &str) -> Result<HashMap<String, (CString, CString)>> {
    fs::read_dir(contract_dir)
        .with_context(|| format!("unable to read directory {}", contract_dir))?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<PathBuf>, _>>()?
        .iter()
        .filter(|path| {
            if let Some(ext) = Path::new(path).extension() {
                ext == "zen"
            } else {
                false
            }
        })
        .map(|path| -> Result<(String, (CString, CString))> {
            let contract = Path::new(path)
                .with_extension("")
                .file_name()
                .with_context(|| format!("unable to get filename for file {:?}", path))?
                .to_str()
                .with_context(|| format!("unable to convert `{:?}` to String", path))?
                .to_string();
            Ok((
                contract,
                (
                    CString::new(
                        fs::read(path)
                            .with_context(|| format!("unable to read file {:?}", path))?,
                    )?,
                    CString::new(
                        fs::read(Path::new(path).with_extension("keys")).with_context(|| {
                            format!("unable to read path {:?}", path.with_extension("keys"))
                        })?,
                    )?,
                ),
            ))
        })
        .collect::<Result<HashMap<String, (CString, CString)>>>()
}

#[post("/contracts/<contract>", format = "json", data = "<msg>")]
fn contracts(
    config: State<ZexecConfig>,
    contracts: State<HashMap<String, (CString, CString)>>,
    contract: String,
    msg: Data,
) -> Result<Option<String>> {
    if let Some((contract, keys)) = &contracts.get(&contract) {
        let mut buf = String::new();
        msg.open()
            .take(BUFFER_LIMIT)
            .read_to_string(&mut buf)
            .context("unable to read from POST data")?;
        let (res, success) = zencode_exec(
            contract.clone(),
            CString::new("")?,
            CString::new(buf)?,
            keys.clone(),
        );
        if success {
            let filename = get_random_name();
            let mut log =
                File::create(Path::new(&config.logs_dir).join(&filename)).with_context(|| {
                    format!(
                        "unable to create file {:?}",
                        Path::new(&config.logs_dir).join(&filename)
                    )
                })?;
            let command = json::parse(&res.output)
                .with_context(|| format!("unable to parse JSON '{}'", res.output))?["command"]
                .as_str()
                .with_context(|| "unable to find command in JSON value")?
                .to_owned();
            let output = Command::new(&command)
                .output()
                .with_context(|| format!("unable to execute command {}", command))?;
            log.write(&output.stdout)
                .with_context(|| "unable to write command output to log")?;
            Ok(Some(filename))
        } else {
            Ok(Some(res.logs))
        }
    } else {
        Ok(None)
    }
}

#[get("/logs/<file..>")]
fn logs(file: PathBuf, config: State<ZexecConfig>) -> Option<NamedFile> {
    NamedFile::open(Path::new(&config.logs_dir).join(file)).ok()
}

fn main() -> Result<()> {
    let opts: Opts = Opts::parse();

    let mut data = [0u8; 32];
    OsRng.fill_bytes(&mut data);
    let secret_key = base64::encode(data);

    let config = Config::build(Environment::Production)
        .address(opts.address)
        .port(opts.port)
        .secret_key(secret_key)
        .finalize()?;

    rocket::custom(config)
        .mount("/", routes![contracts, logs])
        .manage(get_contracts(&opts.contracts_dir)?)
        .manage(ZexecConfig {
            logs_dir: opts.logs_dir,
        })
        .launch();
    Ok(())
}
