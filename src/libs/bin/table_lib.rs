use crate::expression::table::TableData;
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
        getcol, select, headers,
        at, rows, first, last, grep, find, find_last, filter,
        rows_list, first_list, last_list, at_list,
        sort_by,
        append
    })
}
pub fn regist_info() -> BTreeMap<&'static str, BuiltinInfo> {
    reg_info!({
        len => "count rows", "<table>"
        header_len => "count headers", "<table>"
        getcol => "get column by header/index", "<table> <header|index>"
        select => "select columns", "<table> <cols...>"
        headers => "list headers", "<table>"
        rows => "list rows as maps", "<table>"
        first => "get first n row as maps", "<table> [n]"
        last => "get last n row as maps", "<table> [n]"
        at => "get nth row as map", "<table> <index>"
        rows_list => "list rows as lists", "<table>"
        first_list => "get first n row as lists", "<table> [n]"
        last_list => "get last n row as lists", "<table> [n]"
        at_list => "get nth row as list", "<table> <index>"
        grep => "grep rows which contains the string", "<table> <string>"
        find => "find first row index of matching cell", "<table> <cell|fn> [start_index]"
        find_last => "find last row index of matching cell", "<table> <cell|fn> [start_index]"
        filter => "filter rows by condition/cell match", "<table> <cell|fn>"
        sortby => "sort a table by column", "<table> <col>"
        append => "append a row", "<table> <list|set>"
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
    let table = get_table_arg(data, ctx)?;
    Ok(Expression::Integer(table.row_count() as i64))
}
fn header_len(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("header_len", &args, 1, ctx)?;
    let mut it = args.into_iter();
    let data = it.next().unwrap();
    let table = get_table_arg(data, ctx)?;
    Ok(Expression::Integer(table.column_count() as i64))
}
fn getcol(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("get", &args, 2, ctx)?;
    let mut it = args.into_iter();
    let data = it.next().unwrap();
    let table = get_table_arg(data, ctx)?;
    let key = it.next().unwrap();
    let idx = match key {
        Expression::Integer(i) => i as usize,
        Expression::String(s) | Expression::Symbol(s) => table
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
    Ok(table
        .get_column(idx)
        .map_or(Expression::None, Expression::from))
}
pub fn select(
    mut args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_args_len("select", &args, 2.., ctx)?;
    let headers: Vec<String> = match args.split_off(1) {
        s if s.len() == 1 => match s.first().unwrap() {
            Expression::List(list) => list.as_ref().iter().map(|x| x.to_string()).collect(),
            Expression::BSet(list) => list.as_ref().iter().map(|x| x.to_string()).collect(),
            _ => s.iter().map(|x| x.to_string()).collect(),
        },
        s => s.iter().map(|x| x.to_string()).collect(),
    };
    // let data = get_list_ref(&args[0], ctx)?;
    let data_expr = args.into_iter().next().unwrap();
    let data = get_table_arg(data_expr, ctx)?;

    match data.get_columns(&data.get_header_indexes(&headers)) {
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
    let table = get_table_arg(data, ctx)?;
    Ok(Expression::from(table.headers().to_vec()))
}

///every row as a map
fn rows(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("rows", &args, 1, ctx)?;
    let mut it = args.into_iter();
    let data = it.next().unwrap();
    let table = get_table_arg(data, ctx)?;

    Ok(table.to_list_map())
}
fn rows_list(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("rows_list", &args, 1, ctx)?;
    let mut it = args.into_iter();
    let data = it.next().unwrap();
    let table = get_table_arg(data, ctx)?;

    Ok(Expression::from(table.rows().to_vec()))
}

///first row as map
fn first(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_args_len("first", &args, 1..=2, ctx)?;
    let mut it = args.into_iter();
    let data = it.next().unwrap();
    let table = get_table_arg(data, ctx)?;
    match it.next() {
        Some(Expression::Integer(i)) if i > 1 => {
            let r = table
                .rows()
                .iter()
                .take(i as usize)
                .map(|row| {
                    let map: BTreeMap<String, Expression> = table
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
            let row = table.rows().first();
            match row {
                None => Ok(Expression::None),
                Some(row) => {
                    let r = row
                        .iter()
                        .enumerate()
                        .map(|(i, x)| {
                            (
                                table
                                    .headers()
                                    .get(i)
                                    .cloned()
                                    .unwrap_or("unkown".to_string()),
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
fn first_list(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_args_len("first_list", &args, 1..=2, ctx)?;
    let mut it = args.into_iter();
    let data = it.next().unwrap();
    let table = get_table_arg(data, ctx)?;

    match it.next() {
        Some(Expression::Integer(i)) if i > 1 => {
            let r = table
                .rows()
                .iter()
                .take(i as usize)
                .cloned()
                .collect::<Vec<_>>();
            Ok(Expression::from(r))
        }
        _ => {
            let row = table.rows().first();
            match row {
                None => Ok(Expression::None),
                Some(row) => Ok(Expression::from(row.clone())),
            }
        }
    }
}

///last row as map
fn last(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_args_len("last", &args, 1..=2, ctx)?;
    let mut it = args.into_iter();
    let data = it.next().unwrap();
    let table = get_table_arg(data, ctx)?;
    match it.next() {
        Some(Expression::Integer(i)) if i > 1 => {
            let r = table
                .rows()
                .iter()
                .rev()
                .take(i as usize)
                .rev()
                .map(|row| {
                    let map: BTreeMap<String, Expression> = table
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
            let row = table.rows().last();
            match row {
                None => Ok(Expression::None),
                Some(row) => {
                    let r = row
                        .iter()
                        .enumerate()
                        .map(|(i, x)| {
                            (
                                table
                                    .headers()
                                    .get(i)
                                    .cloned()
                                    .unwrap_or("unkown".to_string()),
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
fn last_list(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_args_len("last_list", &args, 1..=2, ctx)?;
    let mut it = args.into_iter();
    let data = it.next().unwrap();
    let table = get_table_arg(data, ctx)?;

    match it.next() {
        Some(Expression::Integer(i)) if i > 1 => {
            let r = table
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
            let row = table.rows().last();
            match row {
                None => Ok(Expression::None),
                Some(row) => Ok(Expression::from(row.clone())),
            }
        }
    }
}

fn at(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("at", &args, 1, ctx)?;
    let mut it = args.into_iter();
    let data = it.next().unwrap();
    let table = get_table_arg(data, ctx)?;
    let index = it.next().unwrap();
    let idx = get_integer_arg(index, ctx)? as usize;

    let row = table.rows().get(idx);
    match row {
        None => Ok(Expression::None),
        Some(row) => {
            let r = row
                .iter()
                .enumerate()
                .map(|(i, x)| {
                    (
                        table
                            .headers()
                            .get(i)
                            .cloned()
                            .unwrap_or("unkown".to_string()),
                        x.clone(),
                    )
                })
                .collect::<BTreeMap<_, _>>();
            Ok(Expression::from(r))
        }
    }
}

fn at_list(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("at_list", &args, 1, ctx)?;
    let mut it = args.into_iter();
    let data = it.next().unwrap();
    let table = get_table_arg(data, ctx)?;
    let index = it.next().unwrap();
    let idx = get_integer_arg(index, ctx)? as usize;

    let row = table.rows().get(idx);
    match row {
        None => Ok(Expression::None),
        Some(row) => Ok(Expression::from(row.clone())),
    }
}

pub fn sort_by(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("sort_by", &args, 2, ctx)?;
    let mut it = args.into_iter();
    let list = it.next().unwrap();
    let key = it.next().unwrap();

    let table = get_table_arg(list, ctx)?;

    let col = match key {
        Expression::Integer(i) => i as usize,
        Expression::String(s) | Expression::Symbol(s) => {
            table.headers().iter().position(|x| x == &s).unwrap_or(0)
        }
        e => {
            return Err(RuntimeError::new(
                RuntimeErrorKind::TypeError {
                    expected: "Integer/String as 2nd arg to sort a table".into(),
                    found: e.type_name(),
                    sym: e.to_string(),
                },
                ctx.clone(),
                0,
            ));
        }
    };
    Ok(Expression::Table(table.sort_by_column(col)))
}

fn grep(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("grep", &args, 2, ctx)?;
    let mut it = args.into_iter();
    let data = it.next().unwrap();
    let table = get_table_arg(data, ctx)?;
    let keyword = it.next().unwrap().to_string();

    let r: Vec<Vec<Expression>> = table
        .rows()
        .iter()
        .filter(|x| x.iter().any(|c| c.to_string().contains(&keyword)))
        .cloned()
        .collect();
    Ok(Expression::from(r))
}

fn find(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_args_len("find", &args, 2..=3, ctx)?;

    let mut it = args.into_iter();
    let data = it.next().unwrap();
    let table = get_table_arg(data, ctx)?;
    let target = it.next().unwrap();
    let start = if let Some(start_expr) = it.next() {
        get_integer_ref(&start_expr, ctx)? as usize
    } else {
        0
    };

    match &target {
        Expression::Function(..) | Expression::Lambda(..) => {
            let state = &mut State::new();
            for (i, row) in table.to_maps().into_iter().enumerate().skip(start) {
                let r = &target.eval_apply(&target, &[row], state, env, 0)?;
                if let Expression::Boolean(true) = r {
                    return Ok(Expression::Integer(i as i64));
                }
            }
            Ok(Expression::None)
        }
        _ => Ok(
            match table
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

fn find_last(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_args_len("find_last", &args, 2..=3, ctx)?;

    let mut it = args.into_iter();
    let data = it.next().unwrap();
    let table = get_table_arg(data, ctx)?;
    let target = it.next().unwrap();
    let start = if let Some(start_expr) = it.next() {
        get_integer_ref(&start_expr, ctx)? as usize
    } else {
        0
    };

    match &target {
        Expression::Function(..) | Expression::Lambda(..) => {
            let state = &mut State::new();
            for (i, row) in table.to_maps().into_iter().enumerate().rev().skip(start) {
                let r = &target.eval_apply(&target, &[row], state, env, 0)?;
                if let Expression::Boolean(true) = r {
                    return Ok(Expression::Integer(i as i64));
                }
            }
            Ok(Expression::None)
        }
        _ => Ok(
            match table
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
    let table = get_table_arg(data, ctx)?;
    let target = it.next().unwrap();

    let result: Vec<Vec<Expression>> = match &target {
        Expression::Function(..) | Expression::Lambda(..) => {
            let state = &mut State::new();
            let r = table
                .to_maps()
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
        _ => table
            .rows()
            .iter()
            .filter(|row| row.iter().any(|col| col == &target))
            .cloned()
            .collect(),
    };
    Ok(Expression::from(result))
}

fn append(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("append", &args, 2, ctx)?;
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

    // if v.len() != t.column_count() {
    //     return Err(RuntimeError::new(
    //         RuntimeErrorKind::CustomError(
    //             format!("length not match while insert:\n`{}`", &val).into(),
    //         ),
    //         ctx.clone(),
    //         0,
    //     ));
    // }
    // let mut rows = t.rows().clone();
    // rows.insert(i as usize, v);
    // let tn = TableData::new(t.headers().to_vec(), rows);

    // } else {
    //     Err(RuntimeError::new(
    //         RuntimeErrorKind::CustomError(
    //             format!("index {} out of bounds for insertion", i).into(),
    //         ),
    //         ctx.clone(),
    //         0,
    //     ))
    // }
}
