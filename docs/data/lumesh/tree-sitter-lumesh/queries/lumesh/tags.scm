; 函数定义
(function_def
name: (symbol) @name) @definition.function

; 函数调用
(function_call
func: (symbol) @name) @reference.call

; 命令表达式
(command_expr
cmd: (symbol) @name) @reference.call

; 变量定义
(normal_assign
target: (symbol) @name) @definition.variable

(multi_assign
target: (symbol) @name) @definition.variable

; 参数定义
(params
param: (symbol) @name) @definition.parameter

(lambda_params
param: (symbol) @name) @definition.parameter

; 模块导入
(use_statement
module: (symbol) @name) @definition.module

(use_statement
alias: (symbol) @name) @definition.module

; 别名定义
(alias_statement
name: (symbol) @name) @definition.variable

; 模块调用
(module_call_expr
func: (symbol) @name) @reference.call

; 变量引用
(symbol) @name @reference.variable

; 局部作用域
(function_def) @local.scope
(lambda_expr) @local.scope
(block) @local.scope

; 局部变量定义
(params
param: (symbol) @local.definition)

(for_expr
variable: (symbol) @local.definition)

(for_expr
index: (symbol) @local.definition)

; 带文档的函数定义
(
(comment)* @doc .
(function_def
name: (symbol) @name) @definition.function
(#select-adjacent! @doc @definition.function)
(#strip! @doc "^##\\s*")
)
