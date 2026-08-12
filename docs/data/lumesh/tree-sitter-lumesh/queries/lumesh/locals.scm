; 函数定义创建新的作用域
(function_def) @local.scope

; Lambda表达式创建新的作用域
(lambda_expr) @local.scope

; 代码块创建新的作用域
(block) @local.scope

; 函数参数定义
(params
param: (symbol) @local.definition)

(params
var_collect: (symbol) @local.definition)

; 函数名定义
(function_def
name: (symbol) @local.definition)

; Lambda参数定义
(lambda_params
param: (symbol) @local.definition)

; for循环变量定义
(for_expr
variable: (symbol) @local.definition)

(for_expr
index: (symbol) @local.definition)

; let声明中的变量定义
(normal_assign
target: (symbol) @local.definition
value: _ @local.definition-value)

(multi_assign
  target: (symbol) @local.definition)

(multi_assign
  value: (_) @local.definition-value)

; 解构赋值中的变量定义
(destruct_list
target: (symbol) @local.definition)

(destruct_map
target: (symbol) @local.definition)

; 别名定义
(alias_statement
name: (symbol) @local.definition)

; 导入模块
(use_statement
module: (symbol) @local.definition)

(use_statement
alias: (symbol) @local.definition)

; 变量引用
(symbol) @local.reference
(variable) @local.reference
