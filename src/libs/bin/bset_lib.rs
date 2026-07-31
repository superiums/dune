use crate::eval::State;
use crate::libs::BuiltinInfo;
use crate::libs::bin::list_lib::clamp;
use crate::libs::helper::*;
use crate::libs::lazy_module::LazyModule;
use crate::{Environment, Expression, RuntimeError, RuntimeErrorKind};
use crate::{reg_info, reg_lazy};
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::rc::Rc;

pub fn regist_lazy() -> LazyModule {
    reg_lazy!({
        // 检查操作
        contains, is_empty, any, all,
        // 数据获取
        first,last,get, len,
        // 查找
        find, filter,
        // 结构修改
        insert, remove, split_first, split_last,
        // 创建操作
        from_list,
        // 集合运算
        union, intersection, difference, symmetric_difference,
        is_subset, is_superset, is_disjoint,
        // 转换操作
        map, to_list,
    })
}

pub fn regist_info() -> BTreeMap<&'static str, BuiltinInfo> {
    reg_info!({
        // 检查操作
        contains => "contains item?", "<set> <item>"
        is_empty => "is empty?", "<set>"
        any => "any item passes fn(item)->bool?", "<set> <fn>"
        all => "all items pass fn(item)->bool?", "<set> <fn>"

        // 数据获取
        first => "smallest item", "<set>"
        last => "largest item", "<set>"
        get => "nth element, negative index from end", "<set> <index>"
        len => "set size", "<set>"

        // 查找
        find => "first item matching fn(item)->bool", "<set> <fn>"
        filter => "keep items where fn(item)->bool", "<set> <fn>"

        // 结构修改
        insert => "add item, returns new set", "<set> <item>"
        remove => "remove item, returns new set", "<set> <item>"
        split_first => "pop smallest, returns [item,rest]", "<set>"
        split_last => "pop largest, returns [item,rest]", "<set>"

        // 创建操作
        from_list => "create set from list", "<list>"

        // 集合运算
        union => "union", "<set1> <set2>"
        intersection => "intersection", "<set1> <set2>"
        difference => "items in set1 not in set2", "<set1> <set2>"
        symmetric_difference => "items in either but not both", "<set1> <set2>"
        is_subset => "set1 ⊆ set2?", "<set1> <set2>"
        is_superset => "set1 ⊇ set2?", "<set1> <set2>"
        is_disjoint => "no common items?", "<set1> <set2>"

        // 转换操作
        map => "apply fn(item)->new_item to each", "<set> <fn>"
        to_list => "to list, sorted order", "<set>"
    })
}

// 检查操作函数
fn contains(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("contains", &args, 2, ctx)?;
    let set = get_bset_ref(&args[0], ctx)?;
    let item = &args[1];

    Ok(Expression::Boolean(set.contains(item)))
}

fn is_empty(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("is_empty", &args, 1, ctx)?;
    let set = get_bset_ref(&args[0], ctx)?;

    Ok(Expression::Boolean(set.is_empty()))
}

// 数据获取函数
fn first(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("first", &args, 1, ctx)?;
    let set = get_bset_ref(&args[0], ctx)?;

    set.as_ref()
        .first()
        .cloned()
        .ok_or_else(|| RuntimeError::common("cannot get first of empty set".into(), ctx.clone(), 0))
}

fn last(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("last", &args, 1, ctx)?;
    let set = get_bset_ref(&args[0], ctx)?;

    set.as_ref()
        .last()
        .cloned()
        .ok_or_else(|| RuntimeError::common("cannot get last of empty set".into(), ctx.clone(), 0))
}
fn get(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("get", &args, 2, ctx)?;
    let set = get_bset_ref(&args[0], ctx)?;
    let n = get_integer_ref(&args[1], ctx)?;
    let index = clamp(n, set.len());

    set.iter().nth(index).cloned().ok_or(RuntimeError::new(
        RuntimeErrorKind::IndexOutOfBounds {
            index: n,
            len: set.len(),
        },
        ctx.clone(),
        0,
    ))
}
fn len(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("len", &args, 1, ctx)?;
    let set = get_bset_ref(&args[0], ctx)?;

    Ok(Expression::Integer(set.len() as i64))
}

// 查找函数
fn find(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("find", &args, 2, ctx)?;

    let predicate = &args[1];
    check_fn_arg(predicate, 1, ctx)?;
    let set = get_bset_ref(&args[0], ctx)?;

    let mut state = State::new();
    for item in set.iter() {
        if predicate
            .eval_apply(predicate, std::slice::from_ref(item), &mut state, env, 0)?
            .is_truthy()
        {
            return Ok(item.clone());
        }
    }

    Ok(Expression::None)
}

fn filter(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("filter", &args, 2, ctx)?;

    let predicate = &args[1];
    check_fn_arg(predicate, 1, ctx)?;
    let set = get_bset_ref(&args[0], ctx)?;

    let mut new_set = BTreeSet::new();
    let mut state = State::new();
    for item in set.iter() {
        if predicate
            .eval_apply(predicate, std::slice::from_ref(item), &mut state, env, 0)?
            .is_truthy()
        {
            new_set.insert(item.clone());
        }
    }

    Ok(Expression::from(new_set))
}

// 结构修改函数
fn insert(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("add", &args, 2, ctx)?;
    let mut it = args.into_iter();
    let set = into_bset(it.next().unwrap(), ctx)?;
    let item = it.next().unwrap();

    let mut new_set = set.as_ref().clone();
    new_set.insert(item);
    Ok(Expression::BSet(Rc::new(new_set)))
}

fn remove(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("remove", &args, 2, ctx)?;
    let mut it = args.into_iter();
    let set = into_bset(it.next().unwrap(), ctx)?;
    let item = it.next().unwrap();

    let mut new_set = set.as_ref().clone();
    new_set.remove(&item);
    Ok(Expression::BSet(Rc::new(new_set)))
}

// 创建操作函数
fn from_list(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("from_items", &args, 1, ctx)?;
    let expr = &args[0];

    if let Expression::List(list) = expr {
        let mut set = BTreeSet::new();
        for item in list.as_ref() {
            set.insert(item.clone());
        }
        Ok(Expression::from(set))
    } else {
        Err(RuntimeError::new(
            RuntimeErrorKind::TypeError {
                expected: "List".into(),
                found: expr.type_name(),
                sym: expr.to_string(),
            },
            ctx.clone(),
            0,
        ))
    }
}

// 集合运算函数
fn union(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("union", &args, 2, ctx)?;
    let set1 = get_bset_ref(&args[0], ctx)?;
    let set2 = get_bset_ref(&args[1], ctx)?;

    let mut new_set = set1.as_ref().clone();
    new_set.extend(set2.iter().cloned());
    Ok(Expression::BSet(Rc::new(new_set)))
}

fn intersection(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("intersect", &args, 2, ctx)?;
    let set1 = get_bset_ref(&args[0], ctx)?;
    let set2 = get_bset_ref(&args[1], ctx)?;

    let new_set = set1.intersection(set2).cloned().collect::<BTreeSet<_>>();
    Ok(Expression::from(new_set))
}

fn difference(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("difference", &args, 2, ctx)?;
    let set1 = get_bset_ref(&args[0], ctx)?;
    let set2 = get_bset_ref(&args[1], ctx)?;

    let new_set = set1.difference(set2).cloned().collect::<BTreeSet<_>>();
    Ok(Expression::from(new_set))
}

fn is_subset(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("is_subset", &args, 2, ctx)?;
    let set1 = get_bset_ref(&args[0], ctx)?;
    let set2 = get_bset_ref(&args[1], ctx)?;

    Ok(Expression::Boolean(set1.is_subset(set2)))
}

fn is_superset(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("is_superset", &args, 2, ctx)?;
    let set1 = get_bset_ref(&args[0], ctx)?;
    let set2 = get_bset_ref(&args[1], ctx)?;

    Ok(Expression::Boolean(set1.is_superset(set2)))
}

// 转换操作函数
fn map(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("map", &args, 2, ctx)?;

    let func = &args[1];
    check_fn_arg(func, 1, ctx)?;
    let set = get_bset_ref(&args[0], ctx)?;

    let mut new_set = BTreeSet::new();
    let mut state = State::new();
    for item in set.iter() {
        let new_item = func.eval_apply(func, std::slice::from_ref(item), &mut state, env, 0)?;
        new_set.insert(new_item);
    }

    Ok(Expression::from(new_set))
}

// 关系判断：与 is_subset/is_superset 组成完整三件套
fn is_disjoint(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("is_disjoint", &args, 2, ctx)?;
    let set1 = get_bset_ref(&args[0], ctx)?;
    let set2 = get_bset_ref(&args[1], ctx)?;
    Ok(Expression::Boolean(set1.is_disjoint(set2)))
}

// 集合运算第四件套：对称差集
fn symmetric_difference(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("symmetric_difference", &args, 2, ctx)?;
    let set1 = get_bset_ref(&args[0], ctx)?;
    let set2 = get_bset_ref(&args[1], ctx)?;
    let new_set = set1
        .symmetric_difference(set2)
        .cloned()
        .collect::<BTreeSet<_>>();
    Ok(Expression::from(new_set))
}

// 弹出最小元素，返回 [popped, new_set]
fn split_first(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("split_first", &args, 1, ctx)?;
    let set = get_bset_ref(&args[0], ctx)?;
    let mut new_set = set.as_ref().clone();
    let popped = new_set.pop_first().unwrap_or(Expression::None);
    Ok(Expression::List(Rc::new(vec![
        popped,
        Expression::BSet(Rc::new(new_set)),
    ])))
}

// 弹出最大元素，返回 [popped, new_set]
fn split_last(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("split_last", &args, 1, ctx)?;
    let set = get_bset_ref(&args[0], ctx)?;
    let mut new_set = set.as_ref().clone();
    let popped = new_set.pop_last().unwrap_or(Expression::None);
    Ok(Expression::List(Rc::new(vec![
        popped,
        Expression::BSet(Rc::new(new_set)),
    ])))
}

// 补齐 any/all，与 list_lib 保持一致的谓词遍历能力
fn any(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("any", &args, 2, ctx)?;
    let predicate = &args[1];
    check_fn_arg(predicate, 1, ctx)?;
    let set = get_bset_ref(&args[0], ctx)?;
    let mut state = State::new();
    for item in set.iter() {
        if predicate
            .eval_apply(predicate, &vec![item.clone()], &mut state, env, 0)?
            .is_truthy()
        {
            return Ok(Expression::Boolean(true));
        }
    }
    Ok(Expression::Boolean(false))
}

fn all(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("all", &args, 2, ctx)?;
    let predicate = &args[1];
    check_fn_arg(predicate, 1, ctx)?;
    let set = get_bset_ref(&args[0], ctx)?;
    let mut state = State::new();
    for item in set.iter() {
        if !predicate
            .eval_apply(predicate, &vec![item.clone()], &mut state, env, 0)?
            .is_truthy()
        {
            return Ok(Expression::Boolean(false));
        }
    }
    Ok(Expression::Boolean(true))
}

fn to_list(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("to_list", &args, 1, ctx)?;
    let set = get_bset_ref(&args[0], ctx)?;

    let list = set.iter().cloned().collect::<Vec<_>>();
    Ok(Expression::from(list))
}

// 辅助函数
fn get_bset_ref<'a>(
    expr: &'a Expression,
    ctx: &Expression,
) -> Result<&'a Rc<BTreeSet<Expression>>, RuntimeError> {
    match expr {
        Expression::BSet(s) => Ok(s),
        e => Err(RuntimeError::new(
            RuntimeErrorKind::TypeError {
                expected: "Set".into(),
                found: e.type_name(),
                sym: e.to_string(),
            },
            ctx.clone(),
            0,
        )),
    }
}

fn into_bset(expr: Expression, ctx: &Expression) -> Result<Rc<BTreeSet<Expression>>, RuntimeError> {
    match expr {
        Expression::BSet(s) => Ok(s),
        e => Err(RuntimeError::new(
            RuntimeErrorKind::TypeError {
                expected: "BSet".into(),
                found: e.type_name(),
                sym: e.to_string(),
            },
            ctx.clone(),
            0,
        )),
    }
}
