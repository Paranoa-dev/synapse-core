use serde_json::{json, Value};

/// Formats output as either table or JSON.
pub struct Formatter;

impl Formatter {
    /// Format a transaction for table display.
    pub fn format_transaction_table(tx: &Value) -> String {
        let id = tx.get("id").and_then(|v| v.as_str()).unwrap_or("N/A");
        let status = tx.get("status").and_then(|v| v.as_str()).unwrap_or("N/A");
        let amount = tx.get("amount").and_then(|v| v.as_str()).unwrap_or("N/A");
        let asset_code = tx
            .get("asset_code")
            .and_then(|v| v.as_str())
            .unwrap_or("N/A");

        format!(
            "ID\t{}\nStatus\t{}\nAmount\t{}\nAsset\t{}\n",
            id, status, amount, asset_code
        )
    }

    /// Format a transaction for JSON display.
    pub fn format_transaction_json(tx: &Value) -> String {
        serde_json::to_string_pretty(tx).unwrap_or_else(|_| "{}".to_string())
    }

    /// Format output based on the requested format.
    pub fn format(format: &str, data: &Value) -> String {
        match format {
            "json" => Self::format_transaction_json(data),
            "table" | _ => Self::format_transaction_table(data),
        }
    }
}
