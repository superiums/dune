use crate::expression::table::TableData;
use crate::{Environment, Int, libs};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::ops::Range;
use std::rc::Rc;
pub mod alias;
pub mod basic;
pub mod catcher;
pub mod cmd_excutor;
pub mod eval;
pub mod eval2;
pub mod eval3;
pub mod from;
pub mod overop;
pub mod pty;
pub mod table;
pub mod terminal;

use chrono::NaiveDateTime;
use regex_lite::Regex;
#[derive(Clone, PartialEq)]
pub enum Expression {
    // 所有嵌套节点改为Rc包裹
    Group(Rc<Self>),
    BinaryOp(String, Rc<Self>, Rc<Self>),
    UnaryOp(String, Rc<Self>, bool),
    RangeOp(String, Rc<Self>, Rc<Self>, Option<Rc<Self>>),
    Pipe(String, Rc<Self>, Rc<Self>),

    // 基础类型保持原样
    Symbol(String),
    Variable(String),
    Integer(Int),
    Float(f64),
    Bytes(Vec<u8>), // 这个保持值类型，因为Rc<Vec>反而增加复杂度
    String(String),
    StringTemplate(Vec<Self>),
    StringSafe(String),
    RegexDef(String),
    TimeDef(String),
    Regex(LumeRegex),
    Boolean(bool),
    None,

    // 集合类型使用Rc
    List(Rc<Vec<Self>>),
    BSet(Rc<BTreeSet<Self>>),
    HMap(Rc<HashMap<String, Self>>),
    Map(Rc<BTreeMap<String, Self>>),

    // 索引和切片优化
    Index(Rc<Self>, Rc<Self>),
    Property(Rc<Self>, Rc<Self>),

    // 其他变体保持不变
    Del(String),
    Declare(String, Rc<Self>),
    Assign(String, Rc<Self>),
    SetParent(String, Rc<Expression>), // 全局变量设置
    Export(String, Option<Rc<Self>>),  // 导出
    AliasDef(String, Rc<Self>),
    For(String, Option<String>, Rc<Self>, Rc<Self>),
    While(Rc<Self>, Rc<Self>),
    Loop(Rc<Self>),
    Match(Rc<Self>, Rc<Vec<(Vec<Self>, Self)>>),
    If(Rc<Self>, Rc<Self>, Rc<Self>),
    Apply(Rc<Self>, Rc<Vec<Self>>),
    Command(Rc<Self>, Rc<Vec<Self>>),
    SymbolRaw(String),
    Lambda(Vec<String>, Rc<Self>, Option<HashMap<String, Self>>),
    Function(
        String,
        Vec<(String, Option<Self>)>,
        Option<String>,
        Rc<Self>,
        Vec<(String, Option<Vec<Expression>>)>,
    ),
    Return(Rc<Self>),
    Break(Rc<Self>),
    Continue,
    Sequence(Vec<Expression>), // 表达式序列
    Block(Rc<Vec<Self>>),
    Quote(Rc<Self>),
    Catch(Rc<Self>, CatchType, Option<Rc<Self>>),
    Range(Range<Int>, usize),
    DateTime(NaiveDateTime),
    FileSize(FileSize),
    Table(TableData),
    Chain(Rc<Expression>, Vec<ChainCall>), // 链式调用
    PipeMethod(String, Rc<Vec<Self>>),
    DestructureAssign(Vec<DestructurePattern>, Rc<Expression>), // 解构赋值
    Use(Option<String>, String),
    ModuleCall(Vec<String>, Rc<Self>), //模块调用
    Blank,
}

#[derive(Debug, Clone)]
pub struct LumeRegex {
    pub regex: Regex,
}
impl PartialEq for LumeRegex {
    fn eq(&self, other: &LumeRegex) -> bool {
        self.regex.as_str() == other.regex.as_str()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChainCall {
    pub method: String,
    pub args: Vec<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DestructurePattern {
    Identifier(String),
    Renamed((String, String)),
    Rest(String), // ...rest 语法
                  // Nested(Box<Expression>),          // 嵌套解构
                  // Default(String, Box<Expression>), // 默认值
}

#[derive(Debug, Clone, PartialEq)]
pub enum SizeUnit {
    B,
    K,
    M,
    G,
    T,
    P,
    None,
}
impl SizeUnit {
    pub fn from_str(unit: &str) -> Self {
        match unit.to_uppercase().as_str() {
            "B" | "" => SizeUnit::B,
            "K" | "KB" => SizeUnit::K,
            "M" | "MB" => SizeUnit::M,
            "G" | "GB" => SizeUnit::G,
            "T" | "TB" => SizeUnit::T,
            "P" | "PB" => SizeUnit::P,
            _ => SizeUnit::B,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FileSize {
    size: u64, // 文件大小，以字节为单位
    unit: SizeUnit,
}
impl FileSize {
    pub fn new(size: u64, unit: SizeUnit) -> Self {
        Self { size, unit }
    }
    pub fn from(size: u64, unit_str: &str) -> Self {
        Self {
            size,
            unit: SizeUnit::from_str(unit_str),
        }
    }
    pub fn from_float(size: f64, unit_str: &str) -> Self {
        let shift: u32 = match SizeUnit::from_str(unit_str) {
            SizeUnit::None | SizeUnit::B => 0,
            SizeUnit::K => 10,
            SizeUnit::M => 20,
            SizeUnit::G => 30,
            SizeUnit::T => 40,
            SizeUnit::P => 50,
        };
        // 直接转换为字节数存储，避免截断精度
        Self {
            size: (size * (1u64 << shift) as f64) as u64,
            unit: SizeUnit::B,
        }
    }
    pub fn from_bytes(size: u64) -> Self {
        Self {
            size,
            unit: SizeUnit::B,
        }
    }

    pub fn to_bytes(&self) -> u64 {
        let mut size = self.size;
        // 根据单位进行转换
        size <<= match &self.unit {
            SizeUnit::None | SizeUnit::B => 0,
            SizeUnit::K => 10,
            SizeUnit::M => 20,
            SizeUnit::G => 30,
            SizeUnit::T => 40,
            SizeUnit::P => 50,
        };
        size
    }
    pub fn to_human_readable(&self) -> String {
        let size = self.to_bytes();
        // 定义单位基数
        const KB: u64 = 1 << 10;
        const MB: u64 = 1 << 20;
        const GB: u64 = 1 << 30;
        const TB: u64 = 1 << 40;
        const PB: u64 = 1 << 50;
        let units = [
            ("P", PB, 50),
            ("T", TB, 40),
            ("G", GB, 30),
            ("M", MB, 20),
            ("K", KB, 10),
            ("B", 0, 0),
        ];

        // 查找合适的单位
        let (unit_str, _, shift) = units
            .iter()
            .find(|(_, base, _)| size >= *base)
            .unwrap_or(units.last().unwrap());
        let scaled_size = size >> shift;

        // 格式化输出
        if *unit_str == "B" {
            format!("{scaled_size}")
        } else if *unit_str == "K" {
            format!("{scaled_size}K")
        } else {
            let frac_size = (size >> (shift - 10)) & 1023; // 计算小数部分
            format!(
                "{:.2}{}",
                scaled_size as f64 + frac_size as f64 * 0.0009765625,
                unit_str
            )
        }
    }
}
impl PartialOrd for FileSize {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.to_bytes().partial_cmp(&other.to_bytes())
    }
}
impl PartialEq for FileSize {
    fn eq(&self, other: &Self) -> bool {
        self.to_bytes().eq(&other.to_bytes())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CatchType {
    Ignore,
    PrintStd,
    PrintErr,
    PrintOver,
    Terminate,
    Deel,
    ToBoolean,
}

impl PartialOrd for Expression {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // ===== 字符串之间比较 =====
        // 先拦截所有“文本类”变体之间的比较，替代原来十几条两两分支
        if let (Some(a), Some(b)) = (self.as_str(), other.as_str()) {
            return a.partial_cmp(b);
        }

        match (self, other) {
            // ===== 同类型简单比较 =====
            (Self::None, Self::None) => Some(Ordering::Equal),
            (Self::Blank, Self::Blank) => Some(Ordering::Equal),
            // (Self::String(a), Self::String(b)) => a.partial_cmp(b),
            (Self::Bytes(a), Self::Bytes(b)) => a.partial_cmp(b),
            (Self::Boolean(a), Self::Boolean(b)) => a.partial_cmp(b),
            (Self::DateTime(a), Self::DateTime(b)) => a.partial_cmp(b),
            (Self::FileSize(a), Self::FileSize(b)) => a.partial_cmp(b),
            (Self::Table(a), Self::Table(b)) => a.rows().partial_cmp(b.rows()),

            // ===== 数值类型互比 =====
            (Self::Integer(a), Self::Integer(b)) => a.partial_cmp(b),
            (Self::Float(a), Self::Float(b)) => a.partial_cmp(b),
            (Self::Float(a), Self::Integer(b)) => a.partial_cmp(&(*b as f64)),
            (Self::Integer(a), Self::Float(b)) => (&(*a as f64)).partial_cmp(b),

            // ===== 字符串与数字互比 =====
            (Self::String(a), Self::Integer(b)) => {
                if let Ok(ai) = a.parse::<i64>() {
                    ai.partial_cmp(b)
                } else {
                    a.parse::<f64>().ok()?.partial_cmp(&(*b as f64))
                }
            }
            (Self::Integer(a), Self::String(b)) => {
                if let Ok(bi) = b.parse::<i64>() {
                    a.partial_cmp(&bi)
                } else {
                    (*a as f64).partial_cmp(&b.parse::<f64>().ok()?)
                }
            }
            (Self::String(a), Self::Float(b)) => a.parse::<f64>().ok()?.partial_cmp(b),
            (Self::Float(a), Self::String(b)) => a.partial_cmp(&b.parse::<f64>().ok()?),

            // bytes
            (Self::Bytes(a), Self::String(b)) => a.as_slice().partial_cmp(b.as_bytes()),
            (Self::String(a), Self::Bytes(b)) => a.as_bytes().partial_cmp(b.as_slice()),

            // 文件大小与数值比较
            (Self::FileSize(a), Self::String(b)) => {
                a.partial_cmp(&libs::from_size_str(b, other).ok()?)
            }
            (Self::String(a), Self::FileSize(b)) => {
                libs::from_size_str(a, self).ok()?.partial_cmp(b)
            }
            (Self::FileSize(a), Self::Integer(b)) => {
                Some(if *b < 0 {
                    Ordering::Greater // u64 >= 0 > i64 负数
                } else {
                    a.to_bytes().cmp(&(*b as u64))
                })
            }
            (Self::Integer(a), Self::FileSize(b)) => {
                Some(if *a < 0 {
                    Ordering::Less // u64 >= 0 > i64 负数
                } else {
                    (*a as u64).cmp(&b.to_bytes())
                })
            }
            (Self::FileSize(a), Self::Float(b)) => {
                if *b < 0.0 {
                    Some(Ordering::Greater)
                } else {
                    (a.to_bytes() as f64).partial_cmp(b)
                }
            }
            (Self::Float(a), Self::FileSize(b)) => {
                if *a < 0.0 {
                    Some(Ordering::Less)
                } else {
                    a.partial_cmp(&(b.to_bytes() as f64))
                }
            }

            // 时间
            (Self::DateTime(a), Self::Integer(b)) => a.and_utc().timestamp().partial_cmp(b),
            (Self::Integer(a), Self::DateTime(b)) => a.partial_cmp(&b.and_utc().timestamp()),
            (Self::DateTime(a), Self::String(_)) => {
                a.partial_cmp(&libs::from_time_str(other.clone(), other).ok()?)
            }

            (Self::String(_), Self::DateTime(b)) => {
                libs::from_time_str(self.clone(), self).ok()?.partial_cmp(b)
            }
            // ===== 集合类型 =====
            (Self::List(a), Self::List(b)) => a.as_slice().partial_cmp(b.as_slice()),

            (Self::BSet(a), Self::BSet(b)) => a.partial_cmp(b),

            (Self::Map(a), Self::Map(b)) => a.partial_cmp(b),
            (Self::HMap(a), Self::HMap(b)) => {
                let mut keys: Vec<_> = a.keys().chain(b.keys()).collect();
                keys.sort();
                keys.dedup();
                for k in keys {
                    let cmp = a.get(k).partial_cmp(&b.get(k));
                    if cmp != Some(Ordering::Equal) {
                        return cmp;
                    }
                }
                Some(Ordering::Equal)
            }

            // ===== 不同种类不比较 =====
            _ => None,
        }
    }
}

impl Ord for Expression {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other)
            .unwrap_or_else(|| self.type_name().cmp(&other.type_name()))
    }
}

impl Eq for Expression {}

pub enum BoxedIterator {
    Range(std::iter::StepBy<std::ops::Range<Int>>),
    Vec(std::vec::IntoIter<Expression>),
    Map(std::collections::hash_map::IntoIter<String, Expression>),
    MapEntries(std::vec::IntoIter<Expression>),
    // Other(Box<dyn Iterator<Item=Expression>>)
}

impl Iterator for BoxedIterator {
    type Item = Expression;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Range(iter) => iter.next().map(Expression::Integer),
            Self::Vec(iter) => iter.next(),
            Self::Map(iter) => iter
                .next()
                .map(|(k, v)| Expression::from(vec![Expression::String(k), v])),
            Self::MapEntries(iter) => iter.next(),
        }
    }
}
