use datafusion::arrow::array::*;
use datafusion::arrow::datatypes::*;
use datafusion::arrow::record_batch::RecordBatch;
use datafusion::catalog::MemTable;
use datafusion::prelude::*;
use std::io::BufRead;
use std::sync::Arc;

mod parser;
use parser::TableParser;
mod schema;
use schema::TypeInferer;

#[tokio::main]
async fn main() -> datafusion::error::Result<()> {
    let args = std::env::args().collect::<Vec<_>>();
    let query_sql = args[1].clone();

    let lines: Vec<String> = std::io::stdin()
        .lock()
        .lines()
        .map(|line| line.unwrap())
        .collect::<Vec<_>>();
    let parser = TableParser::new();
    let headers = parser.parse_header(&lines[0]);
    let rows = lines[1..]
        .iter()
        .map(|line| parser.parse_row(line))
        .collect::<Vec<_>>();

    let inferer = TypeInferer::new();
    let fields = inferer.infer_fields(&headers, &rows);
    let schema = Arc::new(Schema::new(fields));
    let mut field_arrays: Vec<Arc<dyn Array + 'static>> = Vec::with_capacity(headers.len());
    for (i, field) in schema.fields().iter().enumerate() {
        match field.data_type() {
            DataType::Int64 => {
                field_arrays.push(Arc::new(Int64Array::from(
                    rows.iter()
                        .map(|r| r[i].parse::<i64>().unwrap_or(0))
                        .collect::<Vec<_>>(),
                )));
            }
            DataType::Float64 => {
                field_arrays.push(Arc::new(Float64Array::from(
                    rows.iter()
                        .map(|r| r[i].parse::<f64>().unwrap_or(0.0))
                        .collect::<Vec<_>>(),
                )));
            }
            DataType::Utf8 => {
                field_arrays.push(Arc::new(StringArray::from(
                    rows.iter().map(|r| r[i].to_string()).collect::<Vec<_>>(),
                )));
            }
            DataType::Boolean => {
                field_arrays.push(Arc::new(BooleanArray::from(
                    rows.iter()
                        .map(|r| r[i].parse::<bool>().unwrap_or(false))
                        .collect::<Vec<_>>(),
                )));
            }
            _ => {
                panic!("Unsupported data type: {}", field.data_type());
            }
        }
    }

    let batch = RecordBatch::try_new(schema.clone(), field_arrays)?;

    let ctx = SessionContext::new();
    let mem_table = MemTable::try_new(schema, vec![vec![batch]])?;
    ctx.register_table("stdin", Arc::new(mem_table))?;

    let df = ctx.sql(&query_sql).await?;
    df.show().await?;

    Ok(())
}
