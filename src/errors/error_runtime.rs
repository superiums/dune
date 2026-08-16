use std::{borrow::Cow, collections::BTreeMap};

// ============== 运行时错误部分 ==============
use crate::{Expression, Int, LmError, with_print_ast};
use common_macros::b_tree_map;
use thiserror::Error;

#[derive(Debug)]
pub struct RuntimeError {
    pub kind: RuntimeErrorKind,
    pub context: Expression,
    pub depth: usize,
}
#[derive(Debug, Error)]
pub enum RuntimeErrorKind {
    #[error("type `{0}` is not appliable: {1:?}")]
    CannotApply(String, Expression),
    #[error("symbol `{0}` not defined")]
    SymbolNotDefined(String),
    #[error("symbol `{0}` not defined in module {1}\npath trace: {2}")]
    SymbolNotDefinedInModule(String, String, String),
    #[error("symbol `{0}` is not a module, but a `{1}` in {2}\npath trace: {3}")]
    SymbolNotModule(String, &'static str, Cow<'static, str>, String),
    #[error("command `{0}` failed with args:\n  {1:?}")]
    CommandFailed(String, Vec<Expression>),
    #[error("command `{0}` failed:\n  {1}")]
    CommandFailed2(String, String),
    #[error("attempted to iterate over non-list `{0:?}`")]
    ForNonList(Expression),
    #[error("recursion depth exceeded")]
    RecursionDepth(),
    #[error("permission denied while spawn `{0}`")]
    PermissionDenied(String),
    #[error("program `{0}` not found")]
    ProgramNotFound(String),
    #[error("{0}")]
    CustomError(Cow<'static, str>),
    #[error("redeclaration of `{0}`")]
    Redeclaration(String),
    #[error("undeclared variable: `{0}`")]
    UndeclaredVariable(String),
    #[error("undeclared local variable: `{0}`")]
    UndeclaredLocalVariable(String),
    #[error("iterator on `{0}` exhausted, last value: `{1}`")]
    IteratorExhausted(String, String),
    #[error("iter on none iterable")]
    IterOnNoneIterable(),
    #[error("no matching branch while evaluating `{0}`")]
    NoMatchingBranch(String),
    #[error("too many arguments for function `{name}`: max {max}, found {received}")]
    TooManyArguments {
        name: String,
        max: usize,
        received: usize,
    },
    #[error("arguments mismatch for function `{name}`: expected {expected}, found {received}")]
    ArgumentMismatch {
        name: String,
        expected: usize,
        received: usize,
    },
    #[error("invalid default value `{2}` for argument `{1}` in function `{0}`")]
    InvalidDefaultValue(String, String, Expression),
    #[error("invalid operator `{0}`")]
    InvalidOperator(String),
    #[error("index {index} out of bounds (length {len})")]
    IndexOutOfBounds { index: Int, len: usize },
    #[error("key `{0}` not found in map")]
    KeyNotFound(String),
    #[error("method `{0}` not found in module `{1}`")]
    MethodNotFound(Cow<'static, str>, Cow<'static, str>),
    // #[error("module `{0}` not found")]
    // ModuleNotFound(Cow<'static, str>),
    #[error("module `{0}` not defined in `{1}`\npath trace: {2}")]
    NoModuleDefined(String, String, String),
    #[error("no lib function `{0}` defined for {1} during {2}:\n `{3}`")]
    NoLibDefinedFor(String, Cow<'static, str>, Cow<'static, str>, String),
    #[error("no lib function `{0}` defined for {1}")]
    NoLibDefined(String, Cow<'static, str>),
    #[error("not a callable function: `{0}`")]
    NotAFunction(String),
    #[error("type error, expected `{expected}`, found `{found}`:\n  {sym}")]
    TypeError {
        expected: Cow<'static, str>,
        sym: String,
        found: &'static str,
    },
    #[error("illegal return outside function")]
    EarlyReturn(Expression),
    #[error("illegal break outside loop")]
    EarlyBreak(Expression),
    #[error("illegal continue outside loop")]
    EarlyContinue,
    #[error("overflowed when: `{0}`")]
    Overflow(String),
    #[error("wildcard not matched: `{0}`")]
    WildcardNotMatched(String),
    #[error("builtin func `{0}` failed:\n  {1}")]
    BuiltinFailed(String, String),
    #[error("terminated")]
    Terminated,
    #[error("exited with code {0}")]
    Exited(u8),
    #[error("IO Error during {operation}:\n  {kind}: {message}")]
    IoDetailed {
        operation: Cow<'static, str>,
        message: String,
        kind: std::io::ErrorKind,
        os_error: Option<i32>,
    },
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Builtin(#[from] Box<LmError>),
}

impl RuntimeError {
    #[inline]
    pub fn new(kind: RuntimeErrorKind, context: Expression, depth: usize) -> Self {
        Self {
            kind,
            context,
            depth,
        }
    }
    // pub fn from_io_error(io_err: std::io::Error, context: Expression, depth: usize) -> Self {
    //     Self::new(RuntimeErrorKind::Io(io_err), context, depth)
    // }
    pub fn from_io_error(
        io_err: std::io::Error,
        operation: Cow<'static, str>,
        context: Expression,
        depth: usize,
    ) -> Self {
        Self::new(
            RuntimeErrorKind::IoDetailed {
                operation,
                message: io_err.to_string(),
                kind: io_err.kind(),
                os_error: io_err.raw_os_error(),
            },
            context,
            depth,
        )
    }
    #[inline]
    pub fn common(msg: Cow<'static, str>, context: Expression, depth: usize) -> Self {
        Self {
            kind: RuntimeErrorKind::CustomError(msg),
            context,
            depth,
        }
    }
}
const BLUE_START: &str = "\x1b[34m";
const DIM_START: &str = "\x1b[2m";
const RESET: &str = "\x1b[m\x1b[0m";
impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // 使用 RuntimeErrorKind 的 Display 实现
        writeln!(
            f,
            "{}Message   [{}]{}: {}",
            BLUE_START, self.depth, RESET, self.kind
        )?;
        writeln!(
            f,
            "{}Expression[{}]{}: {}",
            BLUE_START, self.depth, RESET, self.context,
        )?;
        if with_print_ast(|b| b) {
            writeln!(
                f,
                "{}SyntaxTree[{}]{}: {}{:?}{}",
                BLUE_START, self.depth, RESET, DIM_START, self.context, RESET
            )?;
        }
        Ok(())
    }
}

impl std::error::Error for RuntimeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.kind.source()
    }
}

impl RuntimeError {
    pub const ERROR_CODE_CANNOT_APPLY: u8 = 101;
    pub const ERROR_CODE_SYMBOL_NOT_DEFINED: u8 = 102;
    pub const ERROR_CODE_COMMAND_FAILED: u8 = 103;
    pub const ERROR_CODE_FOR_NON_LIST: u8 = 105;
    pub const ERROR_CODE_RECURSION_DEPTH: u8 = 106;
    pub const ERROR_CODE_PERMISSION_DENIED: u8 = 107;
    pub const ERROR_CODE_PROGRAM_NOT_FOUND: u8 = 108;
    pub const ERROR_CODE_CUSTOM_ERROR: u8 = 109;
    pub const ERROR_CODE_REDECLARATION: u8 = 110;
    pub const ERROR_CODE_UNDECLARED_VARIABLE: u8 = 111;
    pub const ERROR_CODE_NO_MATCHING_BRANCH: u8 = 112;
    pub const ERROR_CODE_TOO_MANY_ARGUMENTS: u8 = 113;
    pub const ERROR_CODE_ARGUMENT_MISMATCH: u8 = 114;
    pub const ERROR_CODE_INVALID_DEFAULT_VALUE: u8 = 115;
    pub const ERROR_CODE_INVALID_OPERATOR: u8 = 116;
    pub const ERROR_CODE_INDEX_OUT_OF_BOUNDS: u8 = 117;
    pub const ERROR_CODE_KEY_NOT_FOUND: u8 = 118;
    pub const ERROR_CODE_TYPE_ERROR: u8 = 119;
    pub const ERROR_CODE_EARLY_RETURN: u8 = 120;

    pub fn codes() -> BTreeMap<String, Expression> {
        b_tree_map! {
            String::from("cannot_apply") => Expression::from(Self::ERROR_CODE_CANNOT_APPLY),
            String::from("symbol_not_defined") => Expression::from(Self::ERROR_CODE_SYMBOL_NOT_DEFINED),
            String::from("command_failed") => Expression::from(Self::ERROR_CODE_COMMAND_FAILED),
            String::from("for_non_list") => Expression::from(Self::ERROR_CODE_FOR_NON_LIST),
            String::from("recursion_depth") => Expression::from(Self::ERROR_CODE_RECURSION_DEPTH),
            String::from("permission_denied") => Expression::from(Self::ERROR_CODE_PERMISSION_DENIED),
            String::from("program_not_found") => Expression::from(Self::ERROR_CODE_PROGRAM_NOT_FOUND),
            String::from("custom_error") => Expression::from(Self::ERROR_CODE_CUSTOM_ERROR),
            String::from("redeclaration") => Expression::from(Self::ERROR_CODE_REDECLARATION),
            String::from("undeclared_variable") => Expression::from(Self::ERROR_CODE_UNDECLARED_VARIABLE),
            String::from("no_matching_branch") => Expression::from(Self::ERROR_CODE_NO_MATCHING_BRANCH),
            String::from("too_many_arguments") => Expression::from(Self::ERROR_CODE_TOO_MANY_ARGUMENTS),
            String::from("argument_mismatch") => Expression::from(Self::ERROR_CODE_ARGUMENT_MISMATCH),
            String::from("invalid_default_value") => Expression::from(Self::ERROR_CODE_INVALID_DEFAULT_VALUE),
            String::from("invalid_operator") => Expression::from(Self::ERROR_CODE_INVALID_OPERATOR),
            String::from("index_out_of_bounds") => Expression::from(Self::ERROR_CODE_INDEX_OUT_OF_BOUNDS),
            String::from("key_not_found") => Expression::from(Self::ERROR_CODE_KEY_NOT_FOUND),
            String::from("type_error") => Expression::from(Self::ERROR_CODE_TYPE_ERROR),
            String::from("early_return") => Expression::from(Self::ERROR_CODE_EARLY_RETURN),
        }
    }

    pub fn code(&self) -> u8 {
        match self.kind {
            RuntimeErrorKind::CannotApply(..) => Self::ERROR_CODE_CANNOT_APPLY,
            RuntimeErrorKind::SymbolNotDefined(..) => Self::ERROR_CODE_SYMBOL_NOT_DEFINED,
            RuntimeErrorKind::CommandFailed(..) | RuntimeErrorKind::CommandFailed2(..) => {
                Self::ERROR_CODE_COMMAND_FAILED
            }
            RuntimeErrorKind::ForNonList(..) => Self::ERROR_CODE_FOR_NON_LIST,
            RuntimeErrorKind::RecursionDepth(..) => Self::ERROR_CODE_RECURSION_DEPTH,
            RuntimeErrorKind::PermissionDenied(..) => Self::ERROR_CODE_PERMISSION_DENIED,
            RuntimeErrorKind::ProgramNotFound(..) => Self::ERROR_CODE_PROGRAM_NOT_FOUND,
            RuntimeErrorKind::Redeclaration(..) => Self::ERROR_CODE_REDECLARATION,
            RuntimeErrorKind::UndeclaredVariable(..) => Self::ERROR_CODE_UNDECLARED_VARIABLE,
            RuntimeErrorKind::NoMatchingBranch(..) => Self::ERROR_CODE_NO_MATCHING_BRANCH,
            RuntimeErrorKind::TooManyArguments { .. } => Self::ERROR_CODE_TOO_MANY_ARGUMENTS,
            RuntimeErrorKind::ArgumentMismatch { .. } => Self::ERROR_CODE_ARGUMENT_MISMATCH,
            RuntimeErrorKind::InvalidDefaultValue(..) => Self::ERROR_CODE_INVALID_DEFAULT_VALUE,
            RuntimeErrorKind::InvalidOperator(..) => Self::ERROR_CODE_INVALID_OPERATOR,
            RuntimeErrorKind::IndexOutOfBounds { .. } => Self::ERROR_CODE_INDEX_OUT_OF_BOUNDS,
            RuntimeErrorKind::KeyNotFound(..) => Self::ERROR_CODE_KEY_NOT_FOUND,
            RuntimeErrorKind::TypeError { .. } => Self::ERROR_CODE_TYPE_ERROR,
            RuntimeErrorKind::EarlyReturn(..) => Self::ERROR_CODE_EARLY_RETURN,
            _ => Self::ERROR_CODE_CUSTOM_ERROR,
        }
    }
}
