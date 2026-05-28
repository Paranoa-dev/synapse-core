# GraphQL Data Export

## Overview

The GraphQL module provides data export capabilities through the async-graphql schema. This document describes the structure, security considerations, and usage patterns for exporting transaction data.

## Architecture

### Export Resolver

The export functionality is implemented as a GraphQL query resolver that supports multiple output formats:

- **CSV**: Comma-separated values for spreadsheet applications
- **JSON**: Structured JSON for programmatic consumption

### Query Parameters

The export endpoint accepts the following query parameters:

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `format` | String | No | Export format: "csv" or "json" (default: "csv") |
| `from` | String | No | Start date filter (inclusive) - format: YYYY-MM-DD |
| `to` | String | No | End date filter (inclusive) - format: YYYY-MM-DD |
| `status` | String | No | Filter by transaction status (e.g., "pending", "completed") |
| `asset_code` | String | No | Filter by asset code (e.g., "USD", "EUR") |

## Security Considerations

### Input Validation

All export parameters are validated before processing:

1. **Date Format Validation**: Dates must be in YYYY-MM-DD format
2. **Status Validation**: Status values are checked against allowed transaction states
3. **Asset Code Validation**: Asset codes are validated against the configured asset list
4. **Format Validation**: Only "csv" and "json" formats are accepted

### Authentication & Authorization

- All export requests require valid API key authentication via `X-API-Key` header
- Tenant isolation is enforced - users can only export their own transaction data
- Row-level security (RLS) policies prevent cross-tenant data leakage

### Rate Limiting

Export requests are subject to rate limiting:

- Default: 10 requests per minute per tenant
- Large exports may be throttled to prevent resource exhaustion
- Concurrent export requests are limited to prevent database overload

## Usage Examples

### CSV Export

```bash
curl -X GET "http://localhost:3000/export?format=csv&from=2025-01-01&to=2025-01-31" \
  -H "X-API-Key: your-api-key" \
  -H "Accept: text/csv"
```

Response headers:
```
Content-Type: text/csv; charset=utf-8
Content-Disposition: attachment; filename="transactions_2025-01-01_to_2025-01-31.csv"
```

### JSON Export

```bash
curl -X GET "http://localhost:3000/export?format=json&status=completed" \
  -H "X-API-Key: your-api-key" \
  -H "Accept: application/json"
```

### Filtered Export

```bash
curl -X GET "http://localhost:3000/export?format=csv&asset_code=USD&status=pending" \
  -H "X-API-Key: your-api-key"
```

## Performance Optimization

### Streaming

Large exports use streaming to minimize memory usage:

- CSV exports stream rows to the client as they are generated
- JSON exports use streaming arrays for large result sets
- Streaming prevents timeout issues for large datasets

### Indexing

The following database indexes optimize export queries:

- `idx_transactions_created_at`: Speeds up date range filtering
- `idx_transactions_status`: Accelerates status filtering
- `idx_transactions_asset_code`: Optimizes asset code lookups
- `idx_transactions_tenant_id`: Ensures tenant isolation

### Query Optimization

Export queries use:

- Partition pruning for date range queries
- Index-only scans where possible
- Query result caching for repeated exports

## Error Handling

Export requests may return the following errors:

| Status | Error | Description |
|--------|-------|-------------|
| 400 | Invalid Format | Unsupported export format specified |
| 400 | Invalid Date | Date format is not YYYY-MM-DD |
| 400 | Invalid Status | Status value is not recognized |
| 401 | Unauthorized | Missing or invalid API key |
| 429 | Too Many Requests | Rate limit exceeded |
| 500 | Internal Error | Database or processing error |

## Monitoring

### Metrics

The following metrics are tracked for export operations:

- `export_requests_total`: Total number of export requests
- `export_requests_by_format`: Requests grouped by format (csv/json)
- `export_duration_seconds`: Time taken to complete exports
- `export_rows_exported`: Number of rows in each export
- `export_errors_total`: Total export errors by type

### Logging

All export operations are logged with:

- Request parameters (format, filters)
- Tenant ID and user information
- Number of rows exported
- Duration and performance metrics
- Any errors or warnings

## Best Practices

1. **Use Appropriate Filters**: Always specify date ranges to limit result set size
2. **Choose Correct Format**: Use CSV for spreadsheets, JSON for APIs
3. **Handle Large Exports**: Implement pagination or streaming for large datasets
4. **Monitor Rate Limits**: Track rate limit headers in responses
5. **Cache Results**: Cache export results when possible to reduce API calls
6. **Validate Data**: Verify exported data matches expected format and content

## Troubleshooting

### Export Timeout

If exports timeout:

1. Reduce the date range
2. Add more specific filters (status, asset_code)
3. Try JSON format (may be faster for large datasets)
4. Contact support for large historical exports

### Missing Data

If exported data appears incomplete:

1. Verify the date range includes all expected transactions
2. Check that status and asset filters are correct
3. Ensure you have permission to access the data
4. Review audit logs for any filtering applied

### Performance Issues

If exports are slow:

1. Check database query performance with `EXPLAIN ANALYZE`
2. Verify indexes are present and up-to-date
3. Monitor database load during export
4. Consider breaking large exports into smaller chunks
