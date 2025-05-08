use datafusion::arrow::array::*;
use datafusion::arrow::datatypes::*;
use datafusion::arrow::record_batch::RecordBatch;
use datafusion::catalog::MemTable;
use datafusion::prelude::*;
use std::io::BufRead;
use std::sync::Arc;

#[tokio::main]
async fn main() -> datafusion::error::Result<()> {
    let args = std::env::args().collect::<Vec<_>>();
    let query_sql = args[1].clone();

    let lines: Vec<String> = std::io::stdin()
        .lock()
        .lines()
        .map(|line| line.unwrap())
        .collect::<Vec<_>>();
    let headers: Vec<&str> = lines[0].split_whitespace().collect();
    let rows = lines[1..]
        .iter()
        .map(|line| line.split_whitespace().collect::<Vec<&str>>())
        .collect::<Vec<_>>();

    let schema = Arc::new(Schema::new(
        headers
            .iter()
            .map(|header| Field::new(header.to_string(), DataType::Utf8, false))
            .collect::<Vec<_>>(),
    ));
    let mut field_arrays: Vec<Arc<dyn Array + 'static>> = Vec::with_capacity(headers.len());
    for i in 0..headers.len() {
        field_arrays.push(Arc::new(StringArray::from(
            rows.iter().map(|r| r[i].to_string()).collect::<Vec<_>>(),
        )));
    }
    let batch = RecordBatch::try_new(schema.clone(), field_arrays)?;

    let ctx = SessionContext::new();
    let mem_table = MemTable::try_new(schema, vec![vec![batch]])?;
    ctx.register_table("stdin", Arc::new(mem_table))?;

    let df = ctx.sql(&query_sql).await?;
    df.show().await?;

    Ok(())
}
