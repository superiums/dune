use crate::{Expression, RuntimeErrorKind};
use std::fmt;

/// Table
#[derive(Clone, PartialEq)]
pub struct TableData {
    headers: Vec<String>,
    rows: Vec<Vec<Expression>>,
}

impl TableData {
    /// 创建新的表格
    pub fn new(headers: Vec<String>, rows: Vec<Vec<Expression>>) -> Self {
        Self { headers, rows }
    }
    pub fn with_header(headers: Vec<String>) -> Self {
        Self {
            headers,
            rows: Vec::new(),
        }
    }

    /// 添加新行
    pub fn push_row(&mut self, row: Vec<Expression>) {
        // 确保行的列数与表头一致，不足则填充 None
        let mut padded_row = row;
        if padded_row.len() < self.headers.len() {
            padded_row.resize(self.headers.len(), Expression::None);
        } else if padded_row.len() > self.headers.len() {
            // 如果行太长，截断到表头长度
            padded_row.truncate(self.headers.len());
        }
        self.rows.push(padded_row);
    }

    /// 清空并设置数据
    pub fn set_rows(&mut self, rows: Vec<Vec<Expression>>) -> Result<(), RuntimeErrorKind> {
        // 确保行的列数与表头一致
        for row in rows.iter() {
            if row.len() != self.column_count() {
                return Err(RuntimeErrorKind::CustomError(
                    "row size mismatch: {row}".into(),
                ));
            }
        }
        self.rows = rows;
        Ok(())
    }
    pub fn set_rows_vec(&mut self, rows: Vec<Expression>) -> Result<(), RuntimeErrorKind> {
        // 确保行的列数与表头一致
        let rs = rows
            .into_iter()
            .map(|row| match row {
                Expression::List(r) => r.as_ref().clone(),
                Expression::Map(m) => self
                    .headers
                    .iter()
                    .map(|header| m.get(header.as_str()).cloned().unwrap_or(Expression::None))
                    .collect::<Vec<_>>(),
                _ => vec![],
            })
            .collect::<Vec<Vec<_>>>();

        for row in rs.iter() {
            if row.len() != self.column_count() {
                return Err(RuntimeErrorKind::CustomError(
                    "row size mismatch: {row}".into(),
                ));
            }
        }
        self.rows = rs;
        Ok(())
    }

    /// 获取列数据
    pub fn get_column(&self, index: usize) -> Option<Vec<Expression>> {
        if index >= self.headers.len() {
            return None;
        }
        Some(
            self.rows
                .iter()
                .map(|row| row.get(index).cloned().unwrap_or(Expression::None))
                .collect(),
        )
    }
    pub fn column_indexes(&self, col_names: &[String]) -> Vec<usize> {
        col_names
            .iter()
            .filter_map(|x| self.headers.iter().position(|h| h == x))
            .collect()
    }
    pub fn columns(&self, indexes: &[usize]) -> Option<Vec<Vec<Expression>>> {
        if indexes.is_empty() {
            return None;
        }
        Some(
            self.rows
                .iter()
                .map(|row| {
                    indexes
                        .iter()
                        .map(|i| row.get(*i).map_or(Expression::None, |x| x.clone()))
                        .collect()
                })
                .collect::<Vec<_>>(),
        )
    }

    /// 获取行数据
    pub fn get_row(&self, index: usize) -> Option<&[Expression]> {
        self.rows.get(index).map(|row| row.as_slice())
    }

    /// 过滤行
    pub fn filter_rows<F>(&self, mut predicate: F) -> TableData
    where
        F: FnMut(usize, &[Expression]) -> bool,
    {
        let filtered_rows = self
            .rows
            .iter()
            .enumerate()
            .filter_map(|(i, row)| {
                if predicate(i, row.as_slice()) {
                    Some(row.clone())
                } else {
                    None
                }
            })
            .collect();
        TableData {
            headers: self.headers.clone(),
            rows: filtered_rows,
        }
    }

    /// 按列排序
    // pub fn sort_by_column(&mut self, column: usize) {
    //     let mut rows = self.rows.clone();
    //     rows.sort_by(|a, b| match (a.get(column), b.get(column)) {
    //         (Some(a_val), Some(b_val)) => a_val.cmp(b_val),
    //         _ => std::cmp::Ordering::Equal,
    //     });
    //     let _ = self.set_rows(rows);
    // }

    /// 获取表头
    pub fn headers(&self) -> &[String] {
        &self.headers
    }

    pub fn rows(&self) -> &Vec<Vec<Expression>> {
        &self.rows
    }

    /// 获取行数
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    /// 获取列数
    pub fn column_count(&self) -> usize {
        self.headers.len()
    }

    /// 转换为 List<Map> 格式（向后兼容）
    pub fn to_map(&self) -> Expression {
        Expression::from(self.to_map_vec())
    }
    pub fn to_map_vec(&self) -> Vec<Expression> {
        use std::collections::BTreeMap;

        self.rows
            .iter()
            .map(|row| {
                let map: BTreeMap<String, Expression> = self
                    .headers
                    .iter()
                    .enumerate()
                    .map(|(i, header)| {
                        let value = row.get(i).cloned().unwrap_or(Expression::None);
                        (header.clone(), value)
                    })
                    .collect();
                Expression::from(map)
            })
            .collect::<Vec<_>>()
    }
}

impl fmt::Debug for TableData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // 美化格式输出
        if self.headers.is_empty() {
            return write!(f, "[]");
        }

        // 计算每列的最大宽度
        let mut col_widths: Vec<usize> = self.headers.iter().map(|h| h.len()).collect();

        for row in &self.rows {
            for (i, cell) in row.iter().enumerate() {
                if i < col_widths.len() {
                    let cell_str = cell.to_string();
                    col_widths[i] = col_widths[i].max(cell_str.len());
                }
            }
        }

        // 输出表头
        writeln!(
            f,
            "{}",
            "─".repeat(col_widths.iter().sum::<usize>() + col_widths.len() * 3 - 1)
        )?;
        for (i, header) in self.headers.iter().enumerate() {
            if i > 0 {
                write!(f, " │ ")?;
            }
            write!(f, "{:width$}", header, width = col_widths[i])?;
        }
        writeln!(f)?;
        writeln!(
            f,
            "{}",
            "─".repeat(col_widths.iter().sum::<usize>() + col_widths.len() * 3 - 1)
        )?;

        // 输出数据行
        for row in &self.rows {
            for (i, cell) in row.iter().enumerate() {
                if i > 0 {
                    write!(f, " │ ")?;
                }
                write!(f, "{:width$}", cell.to_string(), width = col_widths[i])?;
            }
            writeln!(f)?;
        }

        if !self.rows.is_empty() {
            writeln!(
                f,
                "{}",
                "─".repeat(col_widths.iter().sum::<usize>() + col_widths.len() * 3 - 1)
            )?;
        }
        Ok(())
    }
}
impl fmt::Display for TableData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // 紧凑格式输出
        writeln!(f, "{{")?;

        writeln!(f, "  headers: [")?;
        writeln!(f, "    {}", self.headers.join(", "))?;
        writeln!(f, "  ]\n")?;

        writeln!(f, "  rows: [")?;
        for row in &self.rows {
            writeln!(
                f,
                "    [{}]",
                row.iter()
                    .map(|cell| cell.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            )?;
        }
        writeln!(f, "  ]")?;

        writeln!(f, "}}")
    }
}
