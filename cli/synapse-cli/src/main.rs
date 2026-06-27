use clap::{Parser, Subcommand};
use synapse_cli::{CliConfig, OutputFormat, SynapseCliClient, Formatter};

mod handlers {
    use crate::{CliConfig, OutputFormat, SynapseCliClient, Formatter};
    use super::{TransactionsCmd, SettlementsCmd};

    pub async fn handle_transactions(
        command: TransactionsCmd,
        config: &CliConfig,
        _output_format: OutputFormat,
    ) -> anyhow::Result<()> {
        match command {
            TransactionsCmd::Export {
                format,
                from,
                to,
                status,
                asset_code,
                output,
            } => {
                let client = SynapseCliClient::new(&config.base_url);

                let mut query_params: Vec<(&str, String)> = Vec::new();
                query_params.push(("format", format.clone()));

                let from_owned;
                if let Some(ref f) = from {
                    from_owned = f.clone();
                    query_params.push(("from", from_owned.clone()));
                }

                let to_owned;
                if let Some(ref t) = to {
                    to_owned = t.clone();
                    query_params.push(("to", to_owned.clone()));
                }

                let status_owned;
                if let Some(ref s) = status {
                    status_owned = s.clone();
                    query_params.push(("status", status_owned.clone()));
                }

                let asset_code_owned;
                if let Some(ref ac) = asset_code {
                    asset_code_owned = ac.clone();
                    query_params.push(("asset_code", asset_code_owned.clone()));
                }

                let query_refs: Vec<(&str, &str)> = query_params
                    .iter()
                    .map(|(k, v)| (*k, v.as_str()))
                    .collect();

                let bytes = client.get_bytes("/export", &query_refs).await?;

                if let Some(output_path) = output {
                    std::fs::write(&output_path, &bytes)?;
                    println!("✓ Exported to {}", output_path);
                } else {
                    let output = String::from_utf8(bytes)?;
                    println!("{}", output);
                }

                Ok(())
            }
        }
    }

    pub async fn handle_settlements(
        command: SettlementsCmd,
        config: &CliConfig,
        output_format: OutputFormat,
    ) -> anyhow::Result<()> {
        let client = SynapseCliClient::new(&config.base_url);

        match command {
            SettlementsCmd::List {
                cursor,
                limit,
                direction,
                format,
            } => {
                let mut query_params: Vec<(&str, String)> = Vec::new();
                query_params.push(("limit", limit.to_string()));
                query_params.push(("direction", direction.clone()));

                let cursor_owned;
                if let Some(ref c) = cursor {
                    cursor_owned = c.clone();
                    query_params.push(("cursor", cursor_owned.clone()));
                }

                let query_refs: Vec<(&str, &str)> = query_params
                    .iter()
                    .map(|(k, v)| (*k, v.as_str()))
                    .collect();

                let fmt = OutputFormat::from_str(&format);
                let response: serde_json::Value =
                    client.get_with_query("/settlements", &query_refs).await?;

                let output = Formatter::format_json_output(&response, fmt)?;
                println!("{}", output);

                Ok(())
            }

            SettlementsCmd::Get {
                settlement_id,
                format,
            } => {
                let fmt = OutputFormat::from_str(&format);
                let path = format!("/settlements/{}", settlement_id);
                let response: serde_json::Value = client.get_json(&path).await?;

                let output = Formatter::format_json_output(&response, fmt)?;
                println!("{}", output);

                Ok(())
            }
        }
    }
}

#[derive(Parser)]
#[command(name = "synapse")]
#[command(about = "Synapse CLI - Transaction and Settlement Management")]
#[command(version)]
struct Cli {
    /// Base URL for the Synapse API
    #[arg(long, env = "SYNAPSE_URL")]
    url: Option<String>,

    /// API key for authentication
    #[arg(long, env = "SYNAPSE_API_KEY")]
    api_key: Option<String>,

    /// Output format (table or json)
    #[arg(long, default_value = "table", global = true)]
    format: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Transaction management commands
    Transactions {
        #[command(subcommand)]
        command: TransactionsCmd,
    },

    /// Settlement management commands
    Settlements {
        #[command(subcommand)]
        command: SettlementsCmd,
    },
}

#[derive(Subcommand)]
enum TransactionsCmd {
    /// Export transactions to CSV or JSON format with optional filters.
    ///
    /// The export command streams raw transaction data without parsing or modification.
    /// Output is written to stdout by default, or to a file with --output.
    ///
    /// All filter flags are optional. When omitted, no filter is applied for that dimension.
    ///
    /// Output format:
    /// - CSV (default): Raw comma-separated values with headers, suitable for spreadsheet import
    /// - JSON: Wrapped in a JSON object with metadata, each row as a JSON object
    ///
    /// Example:
    ///   synapse transactions export
    ///   synapse transactions export --format json --status pending
    ///   synapse transactions export --from 2024-01-01 --to 2024-12-31 --output export.csv
    #[command(about = "Export transactions with optional filters")]
    #[command(long_about = "Export transactions to CSV or JSON format with optional filters.\n\n\
                             The export command streams raw transaction data without parsing.\n\
                             Output is written to stdout by default, or to a file with --output.\n\n\
                             All filter flags are optional:\n\n  \
                             * --format: Export format (csv or json, default: csv)\n  \
                             * --from: Start date filter inclusive (YYYY-MM-DD format)\n  \
                             * --to: End date filter inclusive (YYYY-MM-DD format)\n  \
                             * --status: Filter by transaction status (e.g., pending, completed)\n  \
                             * --asset-code: Filter by asset code (e.g., USD, EUR, USDC)\n  \
                             * --output: Save to file instead of stdout")]
    Export {
        /// Export format: 'csv' (default) or 'json'
        /// CSV output contains headers with raw transaction data suitable for spreadsheet import.
        /// JSON output wraps data in a JSON object with optional metadata.
        #[arg(long, default_value = "csv")]
        format: String,

        /// Start date filter (inclusive). Format: YYYY-MM-DD. Optional.
        /// Only transactions created on or after this date are included.
        #[arg(long)]
        from: Option<String>,

        /// End date filter (inclusive). Format: YYYY-MM-DD. Optional.
        /// Only transactions created on or before this date are included.
        #[arg(long)]
        to: Option<String>,

        /// Filter by transaction status. Optional.
        /// Example values: pending, completed, failed, cancelled.
        /// Only transactions with the specified status are included.
        #[arg(long)]
        status: Option<String>,

        /// Filter by asset code. Optional.
        /// Example values: USD, EUR, USDC, BRL.
        /// Only transactions for the specified asset are included.
        #[arg(long)]
        asset_code: Option<String>,

        /// Output file path. Optional. Default: stdout.
        /// If specified, the export is written to this file instead of stdout.
        #[arg(long)]
        output: Option<String>,
    },
}

#[derive(Subcommand)]
enum SettlementsCmd {
    /// List settlements with cursor-based pagination
    List {
        /// Pagination cursor
        #[arg(long)]
        cursor: Option<String>,

        /// Page size (1-100, default 10)
        #[arg(long, default_value = "10")]
        limit: i64,

        /// Pagination direction (forward or backward)
        #[arg(long, default_value = "forward")]
        direction: String,

        /// Output format (table or json)
        #[arg(long, default_value = "table")]
        format: String,
    },

    /// Get a specific settlement by ID
    Get {
        /// Settlement UUID
        settlement_id: String,

        /// Output format (table or json)
        #[arg(long, default_value = "table")]
        format: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("synapse_cli=info".parse()?),
        )
        .init();

    let cli = Cli::parse();

    let mut config = CliConfig::from_env()?;
    if let Some(url) = cli.url {
        config.base_url = url;
    }
    if let Some(api_key) = cli.api_key {
        config.api_key = Some(api_key);
    }

    let output_format = OutputFormat::from_str(&cli.format);

    match cli.command {
        Commands::Transactions { command } => {
            handlers::handle_transactions(command, &config, output_format).await?
        }
        Commands::Settlements { command } => {
            handlers::handle_settlements(command, &config, output_format).await?
        }
    }

    Ok(())
}
