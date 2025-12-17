use clap::Parser;
use greentic_config::{ConfigFileFormat, ConfigLayer, ConfigResolver};
use greentic_runner_host::{RunnerConfig, run as run_host};
use std::path::{Path, PathBuf};

#[derive(Debug, Parser)]
#[command(name = "greentic-runner")]
struct Cli {
    /// Optional path to a greentic config file (toml/json). Overrides project discovery.
    #[arg(long = "config", value_name = "PATH")]
    config: Option<PathBuf>,

    /// Allow dev-only settings in the config (use with caution in prod).
    #[arg(long = "allow-dev")]
    allow_dev: bool,

    /// Print the resolved config and exit.
    #[arg(long = "config-explain")]
    config_explain: bool,

    /// Bindings yaml describing tenant configuration (repeat per tenant)
    #[arg(long = "bindings", value_name = "PATH", required = true)]
    bindings: Vec<PathBuf>,

    /// Port to serve the HTTP server on (default 8080)
    #[arg(long, default_value = "8080")]
    port: u16,
}

#[greentic_types::telemetry::main(service_name = "greentic-runner")]
async fn main() {
    if let Err(err) = run().await {
        tracing::error!(error = %err, "runner failed");
        std::process::exit(1);
    }
}

async fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let (resolver, _) = build_resolver(cli.config.as_deref(), cli.allow_dev)?;
    let resolved = resolver.load()?;
    if cli.config_explain {
        let report =
            greentic_config::explain(&resolved.config, &resolved.provenance, &resolved.warnings);
        println!("{}", report.text);
        return Ok(());
    }
    let cfg = RunnerConfig::from_config(resolved, cli.bindings)?.with_port(cli.port);
    run_host(cfg).await
}

fn build_resolver(
    config_path: Option<&Path>,
    allow_dev: bool,
) -> anyhow::Result<(ConfigResolver, ConfigLayer)> {
    let mut resolver = ConfigResolver::new();
    if allow_dev {
        resolver = resolver.allow_dev(true);
    }
    let layer = if let Some(path) = config_path {
        let format = match path.extension().and_then(|ext| ext.to_str()) {
            Some("json") => ConfigFileFormat::Json,
            _ => ConfigFileFormat::Toml,
        };
        let contents = std::fs::read_to_string(path)?;
        let layer = match format {
            ConfigFileFormat::Toml => toml::from_str(&contents)?,
            ConfigFileFormat::Json => serde_json::from_str(&contents)?,
        };
        if let Some(parent) = path.parent() {
            resolver = resolver.with_project_root_opt(Some(parent.to_path_buf()));
        }
        layer
    } else {
        ConfigLayer::default()
    };
    resolver = resolver.with_cli_overrides(layer.clone());
    Ok((resolver, layer))
}
