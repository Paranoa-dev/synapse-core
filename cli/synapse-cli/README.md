# Synapse CLI

Command-line interface for managing transactions and settlements in Synapse.

## Installation

Build the CLI:

```bash
cargo build --release
```

The binary will be available at `target/release/synapse`.

## Configuration

Set the Synapse API URL and optional API key via environment variables:

```bash
export SYNAPSE_URL=http://localhost:3000
export SYNAPSE_API_KEY=your-api-key
```

Or pass them as command-line flags:

```bash
synapse --url http://localhost:3000 --api-key your-api-key [COMMAND]
```

## Commands

### Transactions

#### Export Transactions

Export transactions to CSV or JSON format with optional filters. The export streams raw data without parsing or modification.

```bash
synapse transactions export [OPTIONS]
```

**Options (all optional):**
- `--format <FORMAT>`: Export format - `csv` (default) or `json`
  - CSV: Raw comma-separated values with headers, suitable for spreadsheet import
  - JSON: Wrapped in a JSON object, each row as a JSON object with metadata
- `--from <FROM>`: Start date filter (inclusive, YYYY-MM-DD format)
- `--to <TO>`: End date filter (inclusive, YYYY-MM-DD format)
- `--status <STATUS>`: Filter by transaction status (e.g., `pending`, `completed`, `failed`, `cancelled`)
- `--asset-code <ASSET_CODE>`: Filter by asset code (e.g., `USD`, `EUR`, `USDC`, `BRL`)
- `--output <OUTPUT>`: Save to file instead of stdout

**Output Format:**

CSV format (default):
```
id,stellar_account,amount,asset_code,status,created_at,updated_at,anchor_transaction_id,callback_type,callback_status
550e8400-e29b-41d4-a716-446655440000,GAAA...,100.00,USD,completed,2024-01-15T10:30:00Z,2024-01-15T11:00:00Z,,send,completed
550e8401-e29b-41d4-a716-446655440001,GBBB...,250.50,EUR,pending,2024-01-15T11:30:00Z,2024-01-15T11:30:00Z,,receive,pending
```

JSON format:
```json
{
  "data": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "stellar_account": "GAAA...",
      "amount": "100.00",
      "asset_code": "USD",
      "status": "completed",
      "created_at": "2024-01-15T10:30:00Z",
      "updated_at": "2024-01-15T11:00:00Z"
    }
  ]
}
```

**Examples:**

Export all transactions as CSV to stdout:
```bash
synapse transactions export
```

Export pending USD transactions as JSON:
```bash
synapse transactions export --format json --status pending --asset-code USD
```

Export transactions from January 2024 as CSV:
```bash
synapse transactions export --from 2024-01-01 --to 2024-01-31
```

Export completed EUR transactions to a file:
```bash
synapse transactions export --status completed --asset-code EUR --output completed_eur.csv
```

Export all EUR and USD transactions in the last 30 days (requires two commands):
```bash
synapse transactions export --asset-code USD --from 2024-01-01 > usd_export.csv
synapse transactions export --asset-code EUR --from 2024-01-01 > eur_export.csv
```

**Notes:**
- The export endpoint streams raw data without intermediate parsing
- Large exports are streamed efficiently without loading entire dataset into memory
- Date filters are inclusive on both ends (from date and to date both included)
- Empty filter results still return valid CSV/JSON with headers (CSV) or empty data array (JSON)
- File output is useful for large exports that may not fit in terminal output

### Settlements

#### List Settlements

List settlements with cursor-based pagination.

```bash
synapse settlements list [OPTIONS]
```

**Options:**
- `--cursor <CURSOR>`: Start from a specific cursor
- `--limit <LIMIT>`: Number of results per page (1-100, default: 10)
- `--direction <DIRECTION>`: Pagination direction - `forward` (default) or `backward`
- `--format <FORMAT>`: Output format - `table` (default) or `json`

**Examples:**

List first 10 settlements:
```bash
synapse settlements list
```

List 50 settlements in JSON format:
```bash
synapse settlements list --limit 50 --format json
```

Navigate with cursor:
```bash
synapse settlements list --cursor <cursor-from-previous-response> --limit 25
```

#### Get Settlement

Get details of a specific settlement.

```bash
synapse settlements get <SETTLEMENT_ID> [OPTIONS]
```

**Arguments:**
- `SETTLEMENT_ID`: The settlement UUID

**Options:**
- `--format <FORMAT>`: Output format - `table` (default) or `json`

**Examples:**

Get settlement details in table format:
```bash
synapse settlements get 550e8400-e29b-41d4-a716-446655440000
```

Get settlement details in JSON format:
```bash
synapse settlements get 550e8400-e29b-41d4-a716-446655440000 --format json
```

## Output Formats

### Table Format (default)
Human-readable output with columns for lists and key-value pairs for objects.

### JSON Format
Full JSON output with all fields, useful for scripting and integration.

## Testing

Run tests:

```bash
cargo test
```

Tests requiring external services are marked with `#[ignore]` and can be run with:

```bash
cargo test -- --ignored
```

## Troubleshooting

### Connection Refused
Ensure the Synapse API server is running and the `--url` or `SYNAPSE_URL` environment variable is correctly set.

### Invalid UUID
Settlement IDs must be valid UUIDs (format: `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx`).

### Empty Results
When exporting transactions or listing settlements returns no results:
- Verify filter parameters are correct
- Check date ranges (use YYYY-MM-DD format)
- Confirm the asset code or status value exists
