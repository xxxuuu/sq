use bpaf::{Parser, construct, long, positional};
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

#[derive(Clone, Debug)]
struct Opts {
    disable_type_infer: bool,
    query_sql: String,
}

#[tokio::main]
async fn main() -> datafusion::error::Result<()> {
    let disable_type_infer = long("disable-type-infer")
        .switch()
        .help("Disable type inference")
        .fallback(false);
    let query_sql = positional::<String>("sql").fallback("SELECT * FROM stdin".to_string());
    let args_parser = construct!(Opts {
        disable_type_infer,
        query_sql
    });
    let opts = args_parser
        .to_options()
        .descr("Query anything with SQL directly in your terminal");
    let opts = opts.run();

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

    let fields = if !opts.disable_type_infer {
        let inferer = TypeInferer::new();
        inferer.infer_fields(&headers, &rows)
    } else {
        headers
            .iter()
            .map(|header| Field::new(header.to_string(), DataType::Utf8, false))
            .collect::<Vec<_>>()
    };
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

    let df = ctx.sql(&opts.query_sql).await?;
    df.show().await?;

    Ok(())
}
