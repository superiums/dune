use std::collections::{BTreeMap, HashMap};
use tabled::{
    Table, Tabled,
    builder::Builder,
    settings::{
        Color, Modify, Style, Width,
        object::{Columns, Rows},
        peaker::PriorityMax,
    },
};

use crate::{Expression, expression::table::TableData, libs::bin::into_lib::strip_ansi_escapes};

/// 嵌套表格能接受的最小可用宽度，低于这个值就没必要画表格了
const MIN_TABLE_WIDTH: usize = 20;

/// 计算多行字符串中"可见"最大宽度（去除 ANSI 转义序列后按字符数计）
fn visible_width(s: &str) -> usize {
    s.lines()
        .map(|l| strip_ansi_escapes(l).chars().count())
        .max()
        .unwrap_or(0)
}

pub fn pretty_printer(arg: &Expression) -> Result<Expression, crate::RuntimeError> {
    let specified_width = crossterm::terminal::size().unwrap_or((120, 0)).0 as usize;
    match arg {
        Expression::Table(table_data) => {
            println!(
                "{}",
                print_table_with_tabled(table_data, true, specified_width)
            )
        }
        Expression::Map(exprs) => println!("{}", pprint_map(exprs.as_ref(), true, specified_width)),
        Expression::HMap(exprs) => {
            println!("{}", pprint_hmap(exprs.as_ref(), true, specified_width))
        }
        Expression::List(exprs) => {
            println!(
                "{}",
                pprint_list(exprs.as_ref(), true, specified_width, false)
            )
        }
        _ => {
            println!("{arg:#}");
        }
    }
    Ok(Expression::None)
}

pub fn pretty_formatter(arg: &Expression) -> String {
    let specified_width = crossterm::terminal::size().unwrap_or((120, 0)).0 as usize;
    match arg {
        Expression::Table(table_data) => {
            print_table_with_tabled(table_data, false, specified_width).to_string()
        }
        Expression::Map(exprs) => pprint_map(exprs.as_ref(), false, specified_width).to_string(),
        Expression::HMap(exprs) => pprint_hmap(exprs.as_ref(), false, specified_width).to_string(),
        Expression::List(exprs) => {
            pprint_list(exprs.as_ref(), false, specified_width, false).to_string()
        }
        _ => format!("{arg:?}"),
    }
}

#[derive(Tabled, PartialEq, Eq, PartialOrd, Ord)]
struct KeyValueRow {
    #[tabled(rename = "KEY")]
    key: String,
    #[tabled(rename = "VALUE")]
    value: String,
}

/// 尝试用子表格渲染，宽度不够时回退成 Display 文本
fn try_render_sub_table<F>(build: F, fallback_val: &Expression, cell_width: usize) -> String
where
    F: FnOnce() -> Table,
{
    // 空间太小，直接不画表格
    if cell_width < MIN_TABLE_WIDTH {
        return textwrap::fill(&format!("{fallback_val}"), cell_width.max(1));
    }

    let sub = build().to_string();

    // 即使按 cell_width 做了 wrap，如果内容本身仍然超宽（例如单个超长 token
    // 或者子表格列太多导致最小宽度也超过 cell_width），就放弃表格形式
    if visible_width(&sub) > cell_width {
        return textwrap::fill(&format!("{fallback_val}"), cell_width);
    }

    sub
}

/// 判断一个 List 是否是"记录列表"（每个元素都是 Map/HMap）
fn is_list_of_records(items: &[Expression]) -> bool {
    !items.is_empty()
        && items
            .iter()
            .all(|e| matches!(e, Expression::Map(_) | Expression::HMap(_)))
}

fn print_table_with_tabled(table: &TableData, with_color: bool, max_width: usize) -> Table {
    let mut builder = Builder::with_capacity(table.row_count(), table.column_count());

    builder.push_record(table.headers());
    for row in table.rows() {
        builder.push_record(row.iter().map(|x| x.to_string()));
    }

    let mut table = builder.build();
    if with_color {
        table.modify(Rows::first(), Color::FG_BLUE);
    }

    table.with(Style::rounded());

    fit_width(&mut table, max_width);
    table
}

/// 类型擦除后的 (key, value) 迭代器。
/// 用 Box<dyn Iterator<...>> 而不是 `impl Iterator<Item=...>` 泛型参数，
/// 是为了打断 pprint_map_internal -> render_field -> render_value -> pprint_map_internal 的
/// 递归单态化：泛型版本每递归一层都会为一个新的具体迭代器类型重新实例化
/// 一份函数体，嵌套深度在类型层面无界，会导致编译期"无限展开"
/// (reached the recursion limit while instantiating)。装箱后签名固定为
/// 同一个具体类型，递归调用不再产生新的单态化实例。
type KvIter<'a> = Box<dyn Iterator<Item = (String, Expression)> + 'a>;

fn pprint_map_internal<'a>(
    items: KvIter<'a>,
    is_hmap: bool,
    with_color: bool,
    max_width: usize,
    nested: bool,
) -> Table {
    const COLS: usize = 2;
    let table_padding = COLS * 3 + 1;
    let available_width = max_width.saturating_sub(table_padding);

    let key_column_width = 12.min(available_width / 2);
    let value_budget = available_width.saturating_sub(key_column_width);

    // tabled::Table::new 必须拿到全部行才能算列宽，加上 is_hmap 要排序，
    // 这一次 Vec 物化无法避免（库限制）；但调用方不再需要事先 collect，
    // 省掉了"调用方 collect 一次 + 这里再 collect 一次"里前面那次。
    let mut rows: Vec<KeyValueRow> = items
        .map(|(key, val)| {
            let value = render_field(&val, value_budget);
            KeyValueRow { key, value }
        })
        .collect();

    if is_hmap {
        rows.sort();
    }
    let mut table = Table::new(rows);

    if is_hmap {
        if with_color {
            table.modify(Columns::first(), Color::FG_BLUE);
        }
        table.modify(Columns::first(), Width::increase(key_column_width));
    } else if with_color {
        table.modify(Columns::first(), Color::FG_GREEN);
    }

    apply_table_style(&mut table, if is_hmap { 1 } else { 0 }, nested);
    fit_width(&mut table, max_width);
    table
}

/// 只有当表格自然宽度超过预算时才强制 wrap；
/// 否则不设置 Width，让 tabled 按内容自身大小渲染（避免被拉伸出空白）。
fn fit_width(table: &mut Table, max_width: usize) {
    table.with(
        Width::wrap(max_width)
            .keep_words(true)
            .priority(PriorityMax::right()),
    );
}

/// 顶层表格保留完整边框；嵌套表格去掉四周边框（只留表头分隔线），
/// 一是视觉上避免"表格套表格"的拥挤感，二是省下两侧竖线占用的宽度，
/// 缓解嵌套时外层单元格里的空白间隙问题。
fn apply_table_style(table: &mut Table, other_style: u8, nested: bool) {
    if nested {
        // Style::psql()：无外框、无竖线，仅表头下一条横线，足够区分表头/数据
        table.with(Style::psql());
    } else if other_style == 1 {
        table.with(Style::markdown());
    } else if other_style == 2 {
        table.with(Style::rounded());
    } else {
        table.with(Style::modern_rounded());
    }
}

fn pprint_map(exprs: &BTreeMap<String, Expression>, with_color: bool, max_width: usize) -> Table {
    pprint_map_internal(
        Box::new(exprs.iter().map(|(k, v)| (k.clone(), v.clone()))),
        false,
        with_color,
        max_width,
        false,
    )
}

pub fn pprint_hmap(
    exprs: &HashMap<String, Expression>,
    with_color: bool,
    max_width: usize,
) -> Table {
    pprint_map_internal(
        Box::new(exprs.iter().map(|(k, v)| (k.clone(), v.clone()))),
        true,
        with_color,
        max_width,
        false,
    )
}

/// 记录中的单个字段：复合类型（可能需要递归成子表格）才用 render_value
/// 的宽度预算逻辑；标量值原样输出完整文本，交给最终 Width::wrap
/// 按列实际内容统一决定是否换行/如何分配宽度，避免"提前按平均值
/// 切碎"导致某些列被过度换行、另一些列却有富余空间。
fn render_field(val: &Expression, cell_width: usize) -> String {
    match val {
        Expression::HMap(_) | Expression::Map(_) => render_value(val, cell_width, false, true),
        Expression::List(items) if is_list_of_records(items) => {
            render_value(val, cell_width, false, true)
        }
        _ => val.to_string(),
    }
}

fn render_value(val: &Expression, cell_width: usize, with_color: bool, nested: bool) -> String {
    match val {
        Expression::HMap(m) => try_render_sub_table(
            || {
                pprint_map_internal(
                    Box::new(m.iter().map(|(k, v)| (k.clone(), v.clone()))),
                    true,
                    with_color,
                    cell_width,
                    nested,
                )
            },
            val,
            cell_width,
        ),
        Expression::Map(m) => try_render_sub_table(
            || {
                pprint_map_internal(
                    Box::new(m.iter().map(|(k, v)| (k.clone(), v.clone()))),
                    false,
                    with_color,
                    cell_width,
                    nested,
                )
            },
            val,
            cell_width,
        ),
        Expression::List(items) if is_list_of_records(items) => try_render_sub_table(
            || pprint_list(items, with_color, cell_width, nested),
            val,
            cell_width,
        ),
        _ => textwrap::fill(&format!("{val}"), cell_width),
    }
}

fn pprint_list(exprs: &[Expression], with_color: bool, max_width: usize, nested: bool) -> Table {
    let (rows, heads_opt) = TableRow {
        rows: exprs,
        max_width,
        col_padding: 5,
    }
    .split_into_rows();

    if rows.is_empty() {
        return Table::default();
    }
    let mut builder;

    let has_header = match heads_opt {
        Some(heads) => {
            builder = Builder::with_capacity(rows.len(), heads.len());
            builder.insert_record(0, heads);
            true
        }
        _ => {
            builder = Builder::with_capacity(rows.len(), rows[0].len());
            false
        }
    };
    for row in rows {
        builder.push_record(row);
    }

    let mut table = builder.build();

    if has_header {
        if with_color {
            table.modify(Rows::first(), Color::FG_BLUE);
        }
        table.with(
            Modify::new(Rows::first()).with(tabled::settings::format::Format::content(|s| {
                s.to_uppercase()
            })),
        );
    }

    apply_table_style(&mut table, if has_header { 2 } else { 0 }, nested);
    fit_width(&mut table, max_width);
    table
}

struct TableRow<'a> {
    rows: &'a [Expression],
    max_width: usize,
    col_padding: usize,
}

impl<'a> TableRow<'a> {
    fn split_into_rows(&self) -> (Vec<Vec<String>>, Option<Vec<String>>) {
        let mut result = Vec::with_capacity(self.rows.len());

        let heads = match self.rows.first() {
            Some(Expression::List(a)) => {
                Some(a.iter().enumerate().map(|(i, _)| format!("C{i}")).collect())
            }
            Some(Expression::HMap(a)) => Some(a.keys().cloned().collect::<Vec<String>>()),
            Some(Expression::Map(a)) => Some(a.keys().cloned().collect::<Vec<String>>()),
            _ => None,
        };
        let mut cols = heads.as_ref().map_or(0, |h| h.len());
        let mut current_row = Vec::with_capacity(cols);

        if cols > 0 {
            // per_cell_width 仅用作“复合值需要递归建子表时”的预算上限，
            // 不再用来提前截断标量字段——标量字段的实际宽度应由最终的
            // Width::wrap(max_width) 统一、按列实际内容智能分配，而不是
            // 建表前就被平均切分打断。
            let per_cell_width = (self.max_width / cols.max(1)).saturating_sub(self.col_padding);

            for expr in self.rows.iter() {
                match expr {
                    Expression::List(a) => {
                        for c in a.iter() {
                            current_row.push(render_field(c, per_cell_width));
                        }
                    }
                    Expression::HMap(a) => {
                        for (_, v) in a.iter() {
                            current_row.push(render_field(v, per_cell_width));
                        }
                    }
                    Expression::Map(a) => {
                        for (_, v) in a.iter() {
                            current_row.push(render_field(v, per_cell_width));
                        }
                    }
                    other => current_row.push(other.to_string()),
                };
                if !current_row.is_empty() {
                    result.push(current_row);
                    current_row = vec![];
                }
            }
            return (result, heads);
        }

        // 一唯表格
        let mut current_len = 0;
        for (i, expr) in self.rows.iter().enumerate() {
            let col = match expr {
                Expression::List(a) => a
                    .as_ref()
                    .iter()
                    .map(|f| f.to_string())
                    .collect::<Vec<String>>()
                    .join(", "),
                Expression::HMap(a) => a
                    .as_ref()
                    .values()
                    .map(|v| v.to_string())
                    .collect::<Vec<String>>()
                    .join("\t"),
                Expression::Map(a) => a
                    .as_ref()
                    .values()
                    .map(|v| v.to_string())
                    .collect::<Vec<String>>()
                    .join("\t"),
                other => other.to_string(),
            };
            let col_width = strip_ansi_escapes(&col).chars().count() + self.col_padding;

            // 两种情况需要换行：
            // 1. 当前行已有内容且加入新列会超限
            // 2. 单列宽度已超过总限制（需强制拆分列）
            if cols == 0 {
                if !current_row.is_empty() && current_len + col_width > self.max_width {
                    cols = i;
                    // dbg!(&cols);
                    result.push(current_row);
                    current_row = vec![];
                    current_len = 0;
                }
            } else if i % cols == 0 {
                // dbg!(&i);
                result.push(current_row);
                current_row = vec![];
                current_len = 0;
            }
            // 处理超宽列（需拆分成多段）
            if col_width > self.max_width {
                let chunks = self.split_column(&col);
                for chunk in chunks {
                    if !current_row.is_empty() {
                        result.push(current_row);
                        current_row = vec![];
                    }
                    current_row.push(chunk);
                }
                current_len = current_row.last().map(|s| s.len()).unwrap_or(0);
            } else {
                current_row.push(col);
                current_len += col_width;
            }
        }

        if !current_row.is_empty() {
            result.push(current_row);
        }
        (result, None)
    }

    fn split_column(&self, text: &str) -> Vec<String> {
        let max_chunk = self.max_width.saturating_sub(self.col_padding);
        if max_chunk == 0 {
            return vec![text.to_string()];
        }

        // 使用textwrap进行智能换行，考虑单词边界
        textwrap::wrap(text, max_chunk)
            .into_iter()
            .map(|s| s.to_string())
            .collect()
    }
}
