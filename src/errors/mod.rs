pub mod error_runtime;
pub mod error_syntax;
use std::collections::BTreeMap;

use crate::Expression;
use common_macros::b_tree_map;
use error_runtime::RuntimeError;
use error_syntax::SyntaxError;
use thiserror::Error;

// ============== 顶级错误类型 ==============

#[derive(Debug, Error)]
pub enum LmError {
    #[error(transparent)]
    Syntax(#[from] SyntaxError),
    #[error(transparent)]
    Runtime(#[from] RuntimeError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    CustomError(String),
    #[error("type error, expected {expected}, found {sym}: {found}")]
    TypeError {
        expected: String,
        sym: String,
        found: String,
    },
    #[error("arguments mismatch for builtin `{name}`: expected {expected}, found {received}")]
    ArgumentMismatch {
        name: String,
        expected: usize,
        received: usize,
    },
}

impl LmError {
    pub const ERROR_CODE_RUNTIME_ERROR: u8 = 201;
    pub const ERROR_CODE_SYNTAX_ERROR: u8 = 202;
    pub const ERROR_CODE_IO_ERROR: u8 = 203;
    pub const ERROR_CODE_CS_ERROR: u8 = 204;
    pub const ERROR_CODE_ARGS_ERROR: u8 = 205;
    pub const ERROR_CODE_TYPE_ERROR: u8 = 206;

    pub fn codes() -> BTreeMap<String, Expression> {
        b_tree_map! {
            String::from("runtime_error") => Expression::from(Self::ERROR_CODE_RUNTIME_ERROR),
            String::from("syntax_error") => Expression::from(Self::ERROR_CODE_SYNTAX_ERROR),
            String::from("io_error") => Expression::from(Self::ERROR_CODE_IO_ERROR),
            String::from("custom_error") => Expression::from(Self::ERROR_CODE_CS_ERROR),
            String::from("args_error") => Expression::from(Self::ERROR_CODE_ARGS_ERROR),
            String::from("type_error") => Expression::from(Self::ERROR_CODE_TYPE_ERROR),
        }
    }
    pub fn code(&self) -> u8 {
        match self {
            Self::Syntax(err) => err.code(),
            Self::Runtime(err) => err.code(),
            Self::Io(_) => Self::ERROR_CODE_IO_ERROR,
            Self::CustomError(_) => Self::ERROR_CODE_CS_ERROR,
            Self::ArgumentMismatch { .. } => Self::ERROR_CODE_ARGS_ERROR,
            Self::TypeError { .. } => Self::ERROR_CODE_TYPE_ERROR,
        }
    }
}
