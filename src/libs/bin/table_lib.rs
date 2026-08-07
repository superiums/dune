use crate::expression::table::TableData;
use crate::libs::bin::into_lib::csv as to_csv;
use crate::libs::bin::list_lib;
use crate::libs::helper::{
    check_args_len, check_exact_args_len, get_integer_arg, get_integer_ref, get_table_arg,
};
use crate::libs::lazy_module::LazyModule;
use crate::{
    Environment, Expression, RuntimeError, RuntimeErrorKind, libs::BuiltinInfo, libs::State,
    reg_info, reg_lazy,
};
use std::collections::BTreeMap;

pub fn regist_lazy() -> LazyModule {
    reg_lazy!({
        len, header_len,
        get_column, select, headers,
        get_map, rows_map, first_map, last_map,
        rows, first, last, get,
        grep, position, rposition, filter,
        sort,
        push,

        is_empty,get_cell,slice,from_maps,
        to_csv
    })
}

pub fn regist_info() -> BTreeMap<&'static str, BuiltinInfo> {
    reg_info!({
        len => "row count", "<table>"
        header_len => "column count", "<table>"
        get_column => "column by header/index", "<table> <header|index>"
        select => "select columns", "<table> <cols...>"
        headers => "list headers", "<table>"

        rows_map => "rows as maps", "<table>"
        first_map => "first n rows as maps", "<table> [n=1]"
        last_map => "last n rows as maps", "<table> [n=1]"
        get_map => "nth row as map", "<table> <index>"
        rows => "rows as lists", "<table>"
        first => "first n rows as lists", "<table> [n=1]"
        last => "last n rows as lists", "<table> [n=1]"
        get => "nth row as list", "<table> <index>"

        grep => "rows containing string", "<table> <string>"
        position => "first row index matching cell/fn(row_map)->bool", "<table> <cell|fn> [start=0]"
        rposition => "last row index matching cell/fn(row_map)->bool", "<table> <cell|fn> [start=0]"
        filter => "filter rows by cell/fn(row_map)->bool", "<table> <cell|fn>"
        sort => "sort, optional fn(a,b)->[-1/0/1]. e.g. sort table 'name'", "<list> [key_fn|±key...]"
        // sort_by => "simple sort by column", "<table> <col>"
        push => "append a row", "<table> <list|set>"

        is_empty => "has no rows?", "<table>"
        get_cell => "single cell, negative row index ok", "<table> <row_index> <header|index>"
        slice => "row range as maps [start,end), negative index ok", "<table> <start> <end>"
        from_maps => "build table from maps, headers = union of keys", "<list_of_maps>"

        to_csv => "serialize to CSV", "<table>"
    })
}

fn len(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("len", &args, 1, ctx)?;
    let mut it = args.into_iter();
    let data = it.next().unwrap();
    let t = get_table_arg(data, ctx)?;
    Ok(Expression::Integer(t.row_count() as i64))
}
fn header_len(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("header_len", &args, 1, ctx)?;
    let mut it = args.into_iter();
    let data = it.next().unwrap();
    let t = get_table_arg(data, ctx)?;
    Ok(Expression::Integer(t.column_count() as i64))
}
fn get_column(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("get_column", &args, 2, ctx)?;
    let mut it = args.into_iter();
    let data = it.next().unwrap();
    let t = get_table_arg(data, ctx)?;
    let key = it.next().unwrap();
    let idx = match key {
        Expression::Integer(i) => i as usize,
        Expression::String(s) | Expression::Symbol(s) => t
            .headers()
            .iter()
            .position(|x| x == &s)
            .ok_or(RuntimeError::common(
            format!("column {} not found", &s).into(),
            ctx.clone(),
            0,
        ))?,
        e => {
            return Err(RuntimeError::new(
                RuntimeErrorKind::TypeError {
                    expected: "String/Index as key".into(),
                    found: e.type_name(),
                    sym: e.to_string(),
                },
                ctx.clone(),
                0,
            ));
        }
    };
    Ok(t.get_column(idx).map_or(Expression::None, Expression::from))
}
pub fn select(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_args_len("select", &args, 2.., ctx)?;
    let mut it = args.into_iter();
    let data_expr = it.next().unwrap();
    let data = get_table_arg(data_expr, ctx)?;

    let headers: Vec<String> = match it {
        mut s if s.len() == 1 => match s.next().unwrap() {
            Expression::List(list) => list.as_ref().iter().map(|x| x.to_string()).collect(),
            Expression::BSet(list) => list.as_ref().iter().map(|x| x.to_string()).collect(),
            Expression::String(s) | Expression::Symbol(s) => vec![s],
            other => vec![other.to_string()],
        },
        s => s.map(|x| x.to_string()).collect(),
    };

    match data.columns(&data.column_indexes(&headers)) {
        Some(rows) => Ok(Expression::Table(TableData::new(headers, rows))),
        None => Ok(Expression::None),
    }
}
fn headers(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("headers", &args, 1, ctx)?;
    let mut it = args.into_iter();
    let data = it.next().unwrap();
    let t = get_table_arg(data, ctx)?;
    Ok(Expression::from(t.headers().to_vec()))
}

///every row as a map
fn rows_map(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("rows_map", &args, 1, ctx)?;
    let mut it = args.into_iter();
    let data = it.next().unwrap();
    let t = get_table_arg(data, ctx)?;

    Ok(t.to_map())
}
fn rows(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("rows", &args, 1, ctx)?;
    let mut it = args.into_iter();
    let data = it.next().unwrap();
    let t = get_table_arg(data, ctx)?;

    Ok(Expression::from(t.rows().to_vec()))
}

///first row as map
fn first_map(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_args_len("first_map", &args, 1..=2, ctx)?;
    let mut it = args.into_iter();
    let data = it.next().unwrap();
    let t = get_table_arg(data, ctx)?;
    match it.next() {
        Some(Expression::Integer(i)) if i > 1 => {
            let r = t
                .rows()
                .iter()
                .take(i as usize)
                .map(|row| {
                    let map: BTreeMap<String, Expression> = t
                        .headers()
                        .iter()
                        .enumerate()
                        .map(|(i, header)| {
                            let value = row.get(i).cloned().unwrap_or(Expression::None);
                            (header.clone(), value)
                        })
                        .collect();
                    Expression::from(map)
                })
                .collect::<Vec<_>>();
            return Ok(Expression::from(r));
        }
        _ => {
            let row = t.rows().first();
            match row {
                None => Ok(Expression::None),
                Some(row) => {
                    let r = row
                        .iter()
                        .enumerate()
                        .map(|(i, x)| {
                            (
                                t.headers().get(i).cloned().unwrap_or("unkown".to_string()),
                                x.clone(),
                            )
                        })
                        .collect::<BTreeMap<_, _>>();
                    Ok(Expression::from(r))
                }
            }
        }
    }
}
///first row as list
fn first(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_args_len("first", &args, 1..=2, ctx)?;
    let mut it = args.into_iter();
    let data = it.next().unwrap();
    let t = get_table_arg(data, ctx)?;

    match it.next() {
        Some(Expression::Integer(i)) if i > 1 => {
            let r = t
                .rows()
                .iter()
                .take(i as usize)
                .cloned()
                .collect::<Vec<_>>();
            Ok(Expression::from(r))
        }
        _ => {
            let row = t.rows().first();
            match row {
                None => Ok(Expression::None),
                Some(row) => Ok(Expression::from(row.clone())),
            }
        }
    }
}

///last row as map
fn last_map(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_args_len("last_map", &args, 1..=2, ctx)?;
    let mut it = args.into_iter();
    let data = it.next().unwrap();
    let t = get_table_arg(data, ctx)?;
    match it.next() {
        Some(Expression::Integer(i)) if i > 1 => {
            let r = t
                .rows()
                .iter()
                .rev()
                .take(i as usize)
                .rev()
                .map(|row| {
                    let map: BTreeMap<String, Expression> = t
                        .headers()
                        .iter()
                        .enumerate()
                        .map(|(i, header)| {
                            let value = row.get(i).cloned().unwrap_or(Expression::None);
                            (header.clone(), value)
                        })
                        .collect();
                    Expression::from(map)
                })
                .collect::<Vec<_>>();
            return Ok(Expression::from(r));
        }
        _ => {
            let row = t.rows().last();
            match row {
                None => Ok(Expression::None),
                Some(row) => {
                    let r = row
                        .iter()
                        .enumerate()
                        .map(|(i, x)| {
                            (
                                t.headers().get(i).cloned().unwrap_or("unkown".to_string()),
                                x.clone(),
                            )
                        })
                        .collect::<BTreeMap<_, _>>();
                    Ok(Expression::from(r))
                }
            }
        }
    }
}
///first row as list
fn last(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_args_len("last", &args, 1..=2, ctx)?;
    let mut it = args.into_iter();
    let data = it.next().unwrap();
    let t = get_table_arg(data, ctx)?;

    match it.next() {
        Some(Expression::Integer(i)) if i > 1 => {
            let r = t
                .rows()
                .iter()
                .rev()
                .take(i as usize)
                .rev()
                .cloned()
                .collect::<Vec<_>>();
            Ok(Expression::from(r))
        }
        _ => {
            let row = t.rows().last();
            match row {
                None => Ok(Expression::None),
                Some(row) => Ok(Expression::from(row.clone())),
            }
        }
    }
}

fn get_map(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("get_map", &args, 1, ctx)?;
    let mut it = args.into_iter();
    let data = it.next().unwrap();
    let t = get_table_arg(data, ctx)?;
    let index = it.next().unwrap();
    let idx = get_integer_arg(index, ctx)? as usize;

    let row = t.rows().get(idx);
    match row {
        None => Ok(Expression::None),
        Some(row) => {
            let r = row
                .iter()
                .enumerate()
                .map(|(i, x)| {
                    (
                        t.headers().get(i).cloned().unwrap_or("unkown".to_string()),
                        x.clone(),
                    )
                })
                .collect::<BTreeMap<_, _>>();
            Ok(Expression::from(r))
        }
    }
}

fn get(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("get", &args, 1, ctx)?;
    let mut it = args.into_iter();
    let data = it.next().unwrap();
    let t = get_table_arg(data, ctx)?;
    let index = it.next().unwrap();
    let idx = get_integer_arg(index, ctx)? as usize;

    let row = t.rows().get(idx);
    match row {
        None => Ok(Expression::None),
        Some(row) => Ok(Expression::from(row.clone())),
    }
}

pub fn sort(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_args_len("sort", &args, 1.., ctx)?;
    let mut it = args.into_iter();
    let list = it.next().unwrap();
    let ops = it.collect();

    let mut t = get_table_arg(list, ctx)?;
    let target = t.to_map_vec();

    // sort
    let sorted = list_lib::sort_vec(target, ops, env, ctx)?;

    t.set_rows_vec(sorted).map_err(|e| RuntimeError {
        kind: e,
        context: ctx.clone(),
        depth: 0,
    })?;
    Ok(Expression::Table(t))
}
// pub fn sort_by(
//     args: Vec<Expression>,
//     _env: &mut Environment,
//     ctx: &Expression,
// ) -> Result<Expression, RuntimeError> {
//     check_exact_args_len("sort_by", &args, 2, ctx)?;
//     let mut it = args.into_iter();
//     let list = it.next().unwrap();
//     let key = it.next().unwrap();

//     let mut t = get_table_arg(list, ctx)?;

//     let col = match key {
//         Expression::Integer(i) => i as usize,
//         Expression::String(s) | Expression::Symbol(s) => {
//             t.headers().iter().position(|x| x == &s).unwrap_or(0)
//         }
//         e => {
//             return Err(RuntimeError::new(
//                 RuntimeErrorKind::TypeError {
//                     expected: "Integer/String as 2nd arg to sort a t".into(),
//                     found: e.type_name(),
//                     sym: e.to_string(),
//                 },
//                 ctx.clone(),
//                 0,
//             ));
//         }
//     };
//     t.sort_by_column(col);
//     Ok(Expression::Table(t))
// }

fn grep(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("grep", &args, 2, ctx)?;
    let mut it = args.into_iter();
    let data = it.next().unwrap();
    let t = get_table_arg(data, ctx)?;
    let keyword = it.next().unwrap().to_string();

    let r: Vec<Vec<Expression>> = t
        .rows()
        .iter()
        .filter(|x| x.iter().any(|c| c.to_string().contains(&keyword)))
        .cloned()
        .collect();
    Ok(Expression::from(r))
}

fn position(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_args_len("position", &args, 2..=3, ctx)?;

    let mut it = args.into_iter();
    let data = it.next().unwrap();
    let t = get_table_arg(data, ctx)?;
    let target = it.next().unwrap();
    let start = if let Some(start_expr) = it.next() {
        get_integer_ref(&start_expr, ctx)? as usize
    } else {
        0
    };

    match &target {
        Expression::Function(..) | Expression::Lambda(..) => {
            let state = &mut State::new();
            for (i, row) in t.to_map_vec().into_iter().enumerate().skip(start) {
                let r = &target.eval_apply(&target, &[row], state, env, 0)?;
                if let Expression::Boolean(true) = r {
                    return Ok(Expression::Integer(i as i64));
                }
            }
            Ok(Expression::None)
        }
        _ => Ok(
            match t
                .rows()
                .iter()
                .skip(start)
                .position(|x| x.iter().any(|c| c == &target))
            {
                Some(index) => Expression::Integer(index as i64),
                None => Expression::None,
            },
        ),
    }
}

fn rposition(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_args_len("rposition", &args, 2..=3, ctx)?;

    let mut it = args.into_iter();
    let data = it.next().unwrap();
    let t = get_table_arg(data, ctx)?;
    let target = it.next().unwrap();
    let start = if let Some(start_expr) = it.next() {
        get_integer_ref(&start_expr, ctx)? as usize
    } else {
        0
    };

    match &target {
        Expression::Function(..) | Expression::Lambda(..) => {
            let state = &mut State::new();
            for (i, row) in t.to_map_vec().into_iter().enumerate().rev().skip(start) {
                let r = &target.eval_apply(&target, &[row], state, env, 0)?;
                if let Expression::Boolean(true) = r {
                    return Ok(Expression::Integer(i as i64));
                }
            }
            Ok(Expression::None)
        }
        _ => Ok(
            match t
                .rows()
                .iter()
                .rev()
                .skip(start)
                .position(|x| x.iter().any(|c| c == &target))
            {
                Some(index) => Expression::Integer(index as i64),
                None => Expression::None,
            },
        ),
    }
}

fn filter(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("filter", &args, 2, ctx)?;

    let mut it = args.into_iter();
    let data = it.next().unwrap();
    let t = get_table_arg(data, ctx)?;
    let target = it.next().unwrap();

    let result: Vec<Vec<Expression>> = match &target {
        Expression::Function(..) | Expression::Lambda(..) => {
            let state = &mut State::new();
            let r = t
                .to_map_vec()
                .into_iter()
                .filter(|row| {
                    target
                        .eval_apply(&target, &[row.clone()], state, env, 0)
                        .is_ok_and(|r| r.is_truthy())
                })
                // .cloned()
                // .map(Expression::from)
                .collect::<Vec<_>>();
            return Ok(Expression::from(r));
        }
        _ => t
            .rows()
            .iter()
            .filter(|row| row.iter().any(|col| col == &target))
            .cloned()
            .collect(),
    };
    Ok(Expression::from(result))
}

fn push(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("push", &args, 2, ctx)?;
    let mut it = args.into_iter();
    let data = it.next().unwrap();
    let mut t = get_table_arg(data, ctx)?;
    // let idx = it.next().unwrap();
    // let i = get_integer_arg(idx, ctx)?;
    let val = it.next().unwrap();

    // if i as usize <= t.rows().len() {
    let v = match &val {
        Expression::List(vlist) => vlist.as_ref().clone(),
        Expression::BSet(vlist) => vlist.iter().cloned().collect(),
        expr => {
            return Err(RuntimeError::new(
                RuntimeErrorKind::TypeError {
                    expected: "List/Set".into(),
                    sym: expr.to_string(),
                    found: expr.type_name(),
                },
                ctx.clone(),
                0,
            ));
        }
    };
    t.push_row(v);
    Ok(Expression::from(t))
}

// ---- is_empty ----
fn is_empty(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("is_empty", &args, 1, ctx)?;
    let t = get_table_arg(args.into_iter().next().unwrap(), ctx)?;
    Ok(Expression::Boolean(t.rows().is_empty()))
}

// ---- get_cell：单元格精确访问 ----
fn get_cell(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("get_cell", &args, 3, ctx)?;
    let mut it = args.into_iter();
    let data = it.next().unwrap();
    let t = get_table_arg(data, ctx)?;
    let a1 = it.next().unwrap();
    let a2 = it.next().unwrap();

    let row_idx = get_integer_ref(&a1, ctx)?;
    let row_len = t.rows().len();
    let row_i = if row_idx < 0 {
        (row_len as i64 + row_idx).max(0) as usize
    } else {
        row_idx as usize
    };

    let col_i = match &a2 {
        Expression::Integer(i) => *i as usize,
        Expression::String(s) | Expression::Symbol(s) => {
            t.headers().iter().position(|h| h == s).ok_or_else(|| {
                RuntimeError::common(format!("no such column: {s}").into(), ctx.clone(), 0)
            })?
        }
        e => {
            return Err(RuntimeError::new(
                RuntimeErrorKind::TypeError {
                    expected: "String/Integer".into(),
                    sym: e.to_string(),
                    found: e.type_name(),
                },
                ctx.clone(),
                0,
            ));
        }
    };

    t.rows()
        .get(row_i)
        .and_then(|row| row.get(col_i))
        .cloned()
        .ok_or_else(|| {
            RuntimeError::common("row/column index out of bounds".into(), ctx.clone(), 0)
        })
}

// ---- slice：行区间截取，返回 Map 形式（与 rows/first/last 保持一致） ----
fn slice(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("slice", &args, 3, ctx)?;

    let mut it = args.into_iter();
    let data = it.next().unwrap();
    let t = get_table_arg(data, ctx)?;
    let a1 = it.next().unwrap();
    let a2 = it.next().unwrap();

    let len = t.rows().len();
    let clamp = |n: i64| -> usize {
        if n < 0 {
            (len as i64 + n).max(0) as usize
        } else {
            (n as usize).min(len)
        }
    };
    let start = clamp(get_integer_ref(&a1, ctx)?);
    let end = clamp(get_integer_ref(&a2, ctx)?);

    if start >= end {
        return Ok(Expression::from(Vec::<Expression>::new()));
    }

    let headers = t.headers();
    let result = t.rows()[start..end]
        .iter()
        .map(|row| {
            let mut m = BTreeMap::new();
            for (h, v) in headers.iter().zip(row.iter()) {
                m.insert(h.clone(), v.clone());
            }
            Expression::from(m)
        })
        .collect::<Vec<_>>();
    Ok(Expression::from(result))
}

// ---- from_maps：反向构造入口，与 to_maps/rows 对称 ----
fn from_maps(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("from_maps", &args, 1, ctx)?;
    let list = match &args[0] {
        Expression::List(l) => l.as_ref().clone(),
        e => {
            return Err(RuntimeError::new(
                RuntimeErrorKind::TypeError {
                    expected: "List of Map".into(),
                    sym: e.to_string(),
                    found: e.type_name(),
                },
                ctx.clone(),
                0,
            ));
        }
    };

    // 收集所有 key 的并集作为表头，保持首次出现顺序
    let mut headers: Vec<String> = Vec::new();
    let mut maps: Vec<BTreeMap<String, Expression>> = Vec::with_capacity(list.len());
    for item in &list {
        let m = match item {
            Expression::Map(m) => m.as_ref().clone(),
            Expression::HMap(m) => m.as_ref().clone().into_iter().collect(),
            e => {
                return Err(RuntimeError::new(
                    RuntimeErrorKind::TypeError {
                        expected: "Map/HMap".into(),
                        sym: e.to_string(),
                        found: e.type_name(),
                    },
                    ctx.clone(),
                    0,
                ));
            }
        };
        for k in m.keys() {
            if !headers.contains(k) {
                headers.push(k.clone());
            }
        }
        maps.push(m);
    }

    let mut t = TableData::with_header(headers.clone());
    for m in maps {
        let row = headers
            .iter()
            .map(|h| m.get(h).cloned().unwrap_or(Expression::None))
            .collect::<Vec<_>>();
        t.push_row(row);
    }
    Ok(Expression::from(t))
}
