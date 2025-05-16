use std::{cmp, collections::HashSet};

use datafusion::arrow::datatypes::{DataType, Field};

const PEEK_ROWS: usize = 30;
pub struct TypeInferer {}

impl TypeInferer {
    pub fn new() -> Self {
        Self {}
    }

    fn infer_data_type(&self, data: &str) -> DataType {
        if data.parse::<i64>().is_ok() {
            return DataType::Int64;
        }
        if data.parse::<f64>().is_ok() {
            return DataType::Float64;
        }
        if data.parse::<bool>().is_ok() {
            return DataType::Boolean;
        }
        DataType::Utf8
    }

    fn infer_data_type_from_rows(&self, rows: &[&str]) -> (DataType, bool) {
        let mut data_types = Vec::new();
        let mut nullable = false;
        for data in rows {
            if data.is_empty() {
                nullable = true;
                continue;
            }
            data_types.push(self.infer_data_type(data));
        }
        let data_types_set: HashSet<_> = data_types.into_iter().collect();
        if data_types_set.len() == 1 {
            return (data_types_set.into_iter().next().unwrap(), nullable);
        }
        if data_types_set.len() == 2
            && data_types_set.contains(&DataType::Int64)
            && data_types_set.contains(&DataType::Float64)
        {
            return (DataType::Float64, nullable);
        }
        (DataType::Utf8, nullable)
    }

    pub fn infer_fields(&self, headers: &[&str], rows: &Vec<Vec<&str>>) -> Vec<Field> {
        headers
            .iter()
            .enumerate()
            .map(|(i, header)| {
                let peek_rows = rows[..cmp::min(rows.len(), PEEK_ROWS)]
                    .iter()
                    .map(|r| r[i])
                    .collect::<Vec<_>>();
                let (data_type, nullable) = self.infer_data_type_from_rows(&peek_rows);
                Field::new(header.to_string(), data_type, nullable)
            })
            .collect::<Vec<_>>()
    }
}

#[cfg(test)]
mod tests {
    use datafusion::arrow::datatypes::{DataType, Field};

    #[test]
    fn test_infer_fields() {
        let inferer = super::TypeInferer::new();
        let headers = vec!["field_1", "field_2", "field_3", "field_4"];
        let rows = vec![
            vec!["1", "2", "", "true"],
            vec!["4.5", "abc", "6", "false"],
            vec!["7", "8", "9", "true"],
        ];
        let fields = inferer.infer_fields(&headers, &rows);
        let expected_fields = vec![
            Field::new("field_1", DataType::Float64, false),
            Field::new("field_2", DataType::Utf8, false),
            Field::new("field_3", DataType::Int64, true),
            Field::new("field_4", DataType::Boolean, false),
        ];
        assert_eq!(fields.len(), 4);
        for (i, field) in fields.iter().enumerate() {
            assert_eq!(field.name(), expected_fields[i].name());
            assert_eq!(field.data_type(), expected_fields[i].data_type());
            assert_eq!(field.is_nullable(), expected_fields[i].is_nullable());
        }
    }
}
