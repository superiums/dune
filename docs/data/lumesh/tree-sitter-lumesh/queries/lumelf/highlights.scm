; Keywords
; Control flow keywords
[
"if"
"else"
"while"
"for"
"loop"
"match"
"return"
"break"
] @keyword.control

[
"use"
"as"
] @keyword.control.import

; Declaration keywords
[
"fn"
"cmd"
] @keyword.function

; Import/module keywords
[
"let"
"set"
"export"
"alias"
"del"
"map"
] @keyword.directive

; Loop-specific keywords
[
"in"
] @keyword.operator

; Operators
; Assignment operators
[
"="
":="
"+="
"-="
"*="
"/="
] @operator.assignment

; Arithmetic operators
[
"+"
"-"
"*"
"/"
"%"
"^"
] @operator.arithmetic

; Comparison operators
[
"==="
"!=="
"=="
"!="
">"
"<"
">="
"<="
"~:"
"!~:"
] @operator.comparison

; Logical operators
[
"&&"
"||"
; "!"
] @operator.logical

; Pipeline operators
[
"|"
"|>"
; "<<"
">>"
">!"
"|^"
] @operator.pipeline

; Error handling operators
[
"?."
"?+"
"??"
"?>"
"?!"
"?:"
"?~"
] @operator.error

; Special operators
[
"?"
; ":"
"->"
"=>"
] @operator.special

; Punctuation
; Brackets by type
[
"("
")"
] @punctuation.bracket.round

[
"["
"]"
] @punctuation.bracket.square

[
"{{"
"}}"
"{"
"}"
] @punctuation.bracket.curly

; Delimiters by context
[
","
] @punctuation.separator.comma

[
";"
] @punctuation.terminator.semicolon

[
"."
] @punctuation.accessor.dot

[
"@"
] @punctuation.special.at

[
"`"
] @punctuation.definition.template

; [
;   "$"
;   "&"
;   "%"
;   "!"
;   ":"
; ] @punctuation.pipe
; lf special
(lf_cmd_def
  name: (symbol) @function.definition)
(lf_pipe) @operator.pipe
(lf_map
  lf_cmd: (symbol) @function.call)

; Range 运算符
(range_expr
  operator: _ @operator.range)

; Slice 分隔符
(slice_expr
  ":" @punctuation.delimiter)

; Blank 占位符
(blank) @constant.builtin

; Built-in constants
(none) @constant.builtin
(boolean) @constant.builtin.boolean

; Variables with context
(variable) @variable.builtin
(symbol) @identifier

; Function-related identifiers
(function_def
name: (symbol) @function.definition)

(function_call
func: (symbol) @function.call)

(command_expr
cmd: (symbol) @function.command)

(chain_expr
method: (symbol) @function.method)

(pipe_method_expr
method: (symbol) @function.method)

(module_call_expr
func: (symbol) @function.call)

; Parameters and arguments
(params
param: (symbol) @variable.parameter)

(params
var_collect: (symbol) @variable.parameter)

(for_expr
variable: (symbol) @variable.parameter.loop)

(for_expr
index: (symbol) @variable.parameter.loop)

; Property access
(map_entry
key: _ @variable.other.member)

(property_expr
property: (symbol) @attribute)

; label
(normal_assign
target: (symbol) @label)

; namespace
(module_call_expr
module: (symbol) @namespace)

; Numbers
(integer) @constant.numeric
(float) @constant.numeric

; Strings with specific types
(string) @string.quoted.double
(string_raw) @string.quoted.single
(string_regex) @string.regexp
(string_time) @string.special.time
(string_template) @string.template

; Special string contexts
(use_statement
module: (string) @string.special.path)

(use_statement
module: (string_raw) @string.special.path)

(path_arg) @string.special.path

; Module detection in various contexts
(use_statement
module: (symbol) @module)

(chain_expr
object: (symbol) @module
(#match? @module "^[A-Z]"))

(chain_expr
object: (symbol) @variable
(#not-match? @variable "^[A-Z]"))

; Expression types for additional context
(add_sub_expr) @expression.arithmetic
(mul_div_expr) @expression.arithmetic
(power_expr) @expression.arithmetic
(comparison_expr) @expression.comparison
(logical_and_expr) @expression.logical
(logical_or_expr) @expression.logical
(conditional_expr) @expression.conditional
(lambda_expr) @expression.function
(pipe_expr) @expression.pipeline
(range_expr) @expression.range
(catch_expr) @expression.error

; Block structures
(block) @structure.block
(list) @structure.list
(map) @structure.map
(namedmap) @structure.map
(sets) @structure.set
(comment) @comment

; Literals
(blank) @constant.builtin
(file_size_literal) @number.float

; Decorators
(decorator
name: (symbol) @function.definition)

; Destructuring
(destruct_list
target: (symbol) @variable.parameter)
(destruct_map
target: (symbol) @variable.parameter)

; Generic field matching for operators
(_ operator: _ @operator)

; Target assignments
(_ target: _ @variable.assignment)

; Function parameters
(_ param: _ @variable.parameter)
