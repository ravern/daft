use common_error::DaftResult;

use crate::micropartition::{MicroPartition, TableState};

use daft_stats::TableMetadata;

impl MicroPartition {
    pub fn slice(&self, start: usize, end: usize) -> DaftResult<Self> {
        let tables = self.tables_or_read(None)?;
        let mut slices_tables = vec![];
        let mut rows_needed = (end - start).max(0);
        let mut offset_so_far = start;

        for tab in tables.iter() {
            if rows_needed == 0 {
                break;
            }

            let tab_rows = tab.len();
            if offset_so_far > 0 && offset_so_far >= tab_rows {
                offset_so_far -= tab_rows;
                continue;
            }

            if offset_so_far == 0 && rows_needed >= tab_rows {
                slices_tables.push(tab.clone());
                rows_needed += tab_rows;
            } else {
                let new_end = (rows_needed + offset_so_far).min(tab_rows);
                let sliced = tab.slice(offset_so_far, new_end)?;
                offset_so_far = 0;
                rows_needed -= sliced.len();
                slices_tables.push(sliced);
            }
        }

        let new_len = slices_tables.iter().map(|t| t.len()).sum();

        Ok(MicroPartition {
            schema: self.schema.clone(),
            state: TableState::Loaded(slices_tables.into()).into(),
            metadata: TableMetadata { length: new_len },
            statistics: self.statistics.clone(),
        })
    }

    pub fn head(&self, num: usize) -> DaftResult<Self> {
        self.slice(0, num)
    }
}
