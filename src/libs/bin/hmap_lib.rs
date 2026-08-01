use std::rc::Rc;

use crate::eval::State;
use crate::libs::bin::top::{dig, flatten, len};
use crate::libs::helper::{
    check_args_len, check_exact_args_len, check_fn_arg, get_hmap_arg, get_hmap_ref, get_string_arg,
    get_string_ref,
};
use crate::libs::lazy_module::LazyModule;
use crate::{
    Environment, Expression, RuntimeError, RuntimeErrorKind, libs::BuiltinInfo, reg_info, reg_lazy,
};

use std::collections::{BTreeMap, HashMap};

pub fn regist_lazy() -> LazyModule {
    reg_lazy!({
        // from top
        len, flatten, dig,
        // 检查操作
        contains_key, contains_value, is_empty,
        // 数据获取
        get, keys, values,
        // 查找
        find, filter,
        // 结构修改
        insert, set, remove,
        // 创建操作
        from_list,
        // 集合运算
        union, intersection, difference, merge,
        // 转换操作
        map, to_bmap, to_list
    })
}

pub fn regist_info() -> BTreeMap<&'static str, BuiltinInfo> {
    reg_info!({
        // 检查操作
        len => "map size", "<map>"
        contains_key => "has key?", "<map> <key>"
        contains_value => "has value?", "<map> <value>"
        is_empty => "is empty?", "<map>"
        flatten => "flatten nested structure", "<map>"

        // 数据获取
        dig => "get nested value by dot path. e.g. dig m 'a.b.0'", "<map|list|range> <path>"
        get => "value by key", "<map> <key>"
        keys => "list of keys", "<map>"
        values => "list of values", "<map>"

        // 查找
        find => "first pair matching fn(k,v)->bool, returns [k,v]", "<map> <fn>"
        filter => "keep pairs where fn(k,v)->bool", "<map> <fn>"

        // 结构修改
        insert => "insert key-value, returns new map", "<map> <key> <value>"
        set => "set existing key's value, returns new map", "<map> <key> <value>"
        remove => "remove key, returns new map", "<map> <key>"

        // 创建操作
        from_list => "create map from list of [k,v] pairs", "<list>"

        // 集合运算
        union => "combine maps, map2 wins on conflict", "<map1> <map2>"
        intersection => "keys in both, values from map1", "<map1> <map2>"
        difference => "keys in map1 not in map2", "<map1> <map2>"
        merge => "deep merge maps, recurse on nested maps", "<map1> <map2> [<map3>...]"

        // 转换操作
        map => "transform keys/values, fn(k,v)->[k,v]", "<map> <entry_fn>"
        to_list => "to list of [k,v] pairs", "<map>"
        to_bmap => "to BtreeMap (ordered)", "<map>"
    })
}

fn insert(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("insert", &args, 3, ctx)?;
    let mut it = args.into_iter();
    let arr = it.next().unwrap();
    let map = get_hmap_ref(&arr, ctx)?;
    let idx = it.next().unwrap();
    let key = get_string_arg(idx, ctx)?;
    let val = it.next().unwrap();

    let mut result = map.as_ref().clone();
    result.insert(key, val);
    Ok(Expression::from(result))
}

// 检查操作函数
fn is_empty(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("is_empty", &args, 1, ctx)?;
    let map = get_hmap_ref(&args[0], ctx)?;

    Ok(Expression::Boolean(map.is_empty()))
}

fn get(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("get", &args, 2, ctx)?;
    let key = get_string_ref(&args[1], ctx)?.as_str();
    let map = get_hmap_ref(&args[0], ctx)?;

    map.get(key).cloned().ok_or_else(|| {
        RuntimeError::common(
            format!("key '{}' not found in HMap", key).into(),
            ctx.clone(),
            0,
        )
    })
}

fn contains_key(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("contains_key", &args, 2, ctx)?;
    let key = get_string_ref(&args[1], ctx)?.as_str();
    let map = get_hmap_ref(&args[0], ctx)?;

    Ok(Expression::Boolean(map.contains_key(key)))
}

fn contains_value(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("contains_value", &args, 2, ctx)?;
    let map = get_hmap_ref(&args[0], ctx)?;

    Ok(Expression::Boolean(map.values().any(|x| x == &args[1])))
}
// 数据获取函数
fn to_list(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("to_list", &args, 1, ctx)?;
    let map = get_hmap_ref(&args[0], ctx)?;

    let r = map
        .iter()
        .map(|(k, v)| Expression::from(vec![Expression::String(k.clone()), v.clone()]))
        .collect::<Vec<_>>();
    Ok(Expression::from(r))
}

fn keys(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("keys", &args, 1, ctx)?;
    let map = get_hmap_ref(&args[0], ctx)?;

    let r = map
        .keys()
        .map(|k| Expression::String(k.clone()))
        .collect::<Vec<_>>();
    Ok(Expression::from(r))
}

fn values(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("values", &args, 1, ctx)?;
    let map = get_hmap_ref(&args[0], ctx)?;

    let r = map.values().cloned().collect::<Vec<_>>();
    Ok(Expression::from(r))
}

// 查找函数
fn find(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("find", &args, 2, ctx)?;

    let predicate = &args[1];
    check_fn_arg(predicate, 2, ctx)?;
    let map = get_hmap_ref(&args[0], ctx)?;

    let items = map
        .iter()
        .map(|(k, v)| vec![Expression::String(k.clone()), v.clone()])
        .collect::<Vec<_>>();

    let mut state = State::new();
    for it in items {
        if predicate
            .eval_apply(predicate, &it, &mut state, env, 0)?
            .is_truthy()
        {
            return Ok(Expression::from(it));
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
    check_fn_arg(predicate, 2, ctx)?;
    let map = get_hmap_ref(&args[0], ctx)?;

    let items = map
        .iter()
        .map(|(k, v)| vec![Expression::String(k.clone()), v.clone()])
        .collect::<Vec<_>>();

    let mut new_map = HashMap::new();
    let mut state = State::new();
    for it in items {
        if predicate
            .eval_apply(predicate, &it, &mut state, env, 0)?
            .is_truthy()
        {
            new_map.insert(it[0].to_string(), it[1].clone());
        }
    }

    Ok(Expression::from(new_map))
}

// 结构修改函数
fn remove(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("remove", &args, 2, ctx)?;
    let mut it = args.into_iter();
    let map = get_hmap_arg(it.next().unwrap(), ctx)?;
    let key = it.next().unwrap();

    let mut new_map = map.as_ref().clone();
    new_map.remove(&key.to_string());
    Ok(Expression::HMap(Rc::new(new_map)))
}

fn set(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("set", &args, 3, ctx)?;
    let mut it = args.into_iter();
    let map = get_hmap_arg(it.next().unwrap(), ctx)?;
    let key_expr = it.next().unwrap();
    let val_expr = it.next().unwrap();

    let key_str = get_string_arg(key_expr, ctx)?;

    if map.as_ref().contains_key(&key_str) {
        let mut new_map = map.as_ref().clone();
        new_map.insert(key_str, val_expr);
        Ok(Expression::HMap(Rc::new(new_map)))
    } else {
        Err(RuntimeError::common(
            format!("key '{key_str}' not found in hmap").into(),
            ctx.clone(),
            0,
        ))
    }
}

// 创建操作函数
fn from_list(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("from_list", &args, 1, ctx)?;
    let expr = &args[0];

    if let Expression::List(list) = expr {
        let mut map = HashMap::new();
        for item in list.as_ref() {
            if let Expression::List(pair) = item
                && pair.as_ref().len() == 2
            {
                map.insert(pair.as_ref()[0].to_string(), pair.as_ref()[1].clone());
            }
        }
        Ok(Expression::from(map))
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
    let map1 = get_hmap_ref(&args[0], ctx)?;
    let map2 = get_hmap_ref(&args[1], ctx)?;

    let mut new_map = map1.as_ref().clone();
    new_map.extend(map2.as_ref().iter().map(|(k, v)| (k.clone(), v.clone())));
    Ok(Expression::HMap(Rc::new(new_map)))
}

fn intersection(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("intersection", &args, 2, ctx)?;
    let map1 = get_hmap_ref(&args[0], ctx)?;
    let map2 = get_hmap_ref(&args[1], ctx)?;

    let mut new_map = HashMap::new();
    for (k, v) in map1.as_ref() {
        if map2.as_ref().contains_key(k) {
            new_map.insert(k.clone(), v.clone());
        }
    }
    Ok(Expression::from(new_map))
}

fn difference(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("difference", &args, 2, ctx)?;
    let map1 = get_hmap_ref(&args[0], ctx)?;
    let map2 = get_hmap_ref(&args[1], ctx)?;

    let mut new_map = HashMap::new();
    for (k, v) in map1.as_ref() {
        if !map2.as_ref().contains_key(k) {
            new_map.insert(k.clone(), v.clone());
        }
    }
    Ok(Expression::from(new_map))
}

fn merge(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_args_len("merge", &args, 2.., ctx)?;

    let maps = args
        .into_iter()
        .map(|a| get_hmap_arg(a, ctx).unwrap_or(Rc::new(HashMap::new())))
        .collect::<Vec<_>>();

    if maps.is_empty() {
        return Ok(Expression::None);
    }

    let mut it = maps.into_iter();
    let base = it.next().unwrap();
    let mut result = HashMap::new();

    for next in it.skip(1) {
        result = deep_merge_hmaps(base.as_ref(), next.as_ref())?;
    }

    Ok(Expression::from(result))
}

fn deep_merge_hmaps(
    a: &HashMap<String, Expression>,
    b: &HashMap<String, Expression>,
) -> Result<HashMap<String, Expression>, RuntimeError> {
    let mut result = a.clone();

    for (k, v) in b.iter() {
        if let Some(existing) = result.get(k)
            && let (Expression::HMap(ma), Expression::HMap(mb)) = (existing, v)
        {
            result.insert(
                k.clone(),
                Expression::HMap(Rc::new(deep_merge_hmaps(ma.as_ref(), mb.as_ref())?)),
            );
            continue;
        }
        result.insert(k.clone(), v.clone());
    }

    Ok(result)
}

// 转换操作函数
fn map(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("map", &args, 2, ctx)?;

    let map_fn = &args[1];

    check_fn_arg(map_fn, 2, ctx)?;

    let map = get_hmap_ref(&args[0], ctx)?;

    let mut new_map = HashMap::new();
    let mut state = State::new();
    for (k, v) in map.iter() {
        let new_kv = map_fn.eval_apply(
            map_fn,
            &[Expression::String(k.clone()), v.clone()],
            &mut state,
            env,
            0,
        )?;
        match new_kv {
            Expression::List(ls) => {
                new_map.insert(
                    ls.get(0).map_or(k.clone(), |nk| nk.to_string()),
                    ls.get(1).cloned().unwrap_or(Expression::None),
                );
            }
            Expression::Map(nm) => new_map.extend(nm.as_ref().clone()),
            Expression::HMap(nm) => new_map.extend(nm.as_ref().clone()),
            _ => {
                return Err(RuntimeError::common(
                    "map fn should return List/Map".into(),
                    ctx.clone(),
                    0,
                ));
            }
        }
    }

    Ok(Expression::from(new_map))
}

// 转换操作函数
fn to_bmap(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("to_bmap", &args, 1, ctx)?;
    let hmap = get_hmap_ref(&args[0], ctx)?;

    // 将 HashMap 转换为 BTreeMap
    let bmap: BTreeMap<String, Expression> =
        hmap.iter().map(|(k, v)| (k.clone(), v.clone())).collect();

    Ok(Expression::from(bmap))
}
