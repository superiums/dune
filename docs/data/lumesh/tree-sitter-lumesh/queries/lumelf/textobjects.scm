; ---function
; 函数定义
(function_def) @function.around
(function_def
body: (_) @function.inside)
(lf_cmd_def) @function.around
(lf_cmd_body
body: (_) @function.inside)
; (function_def
; name: (symbol) @function.movement)

; Lambda表达式
(lambda_expr) @function.around
(lambda_expr
body: (_) @function.inside)

; 模块调用
(module_call_expr) @function.around
(module_call_expr
arg: (_) @function.inside)

; 代码块
; (block) @block.around
; (block
; statement: (_) @block.inside)

; ---parameter
; 函数调用
(function_call) @parameter.around
(function_call
arg: (_) @parameter.inside)

; 命令表达式
(command_expr) @parameter.around
(command_expr
arg: (_) @parameter.inside)

; 参数列表
(params) @parameter.around
(params
param: (_) @parameter.inside)

; ---test
; 控制流结构
(if_expr) @test.around
(if_expr
then_branch: (_) @test.inside)

; (while_expr) @loop.around
; (while_expr
; body: (_) @loop.inside)

; (for_expr) @loop.around
; (for_expr
; body: (_) @loop.inside)

; (loop_expr) @loop.around
; (loop_expr
; body: (_) @loop.inside)

(match_arm) @test.around
(match_arm
result: (_) @test.inside)

(conditional_expr) @test.around
(conditional_expr
true_expr: (_) @test.inside)

(catch_expr) @test.around
(catch_expr
try: (_) @test.inside)

; 管道表达式
(pipe_expr) @test.around
(pipe_expr
right: (_) @test.inside)

; 装饰器
(decorator) @test.around
(decorator
args: (_) @test.inside)

; ---entry
; 集合类型
(list) @entry.around
(list
element: (_) @entry.inside)

(map) @entry.around
(map
entry: (_) @entry.inside)

(namedmap) @entry.around
(namedmap
entry: (_) @entry.inside)

(sets) @entry.around
(sets
element: (_) @entry.inside)

(match_expr) @entry.around
(match_expr
arm: (_) @entry.inside)

; ---comment
; 注释
(comment) @comment.inside
(comment)+ @comment.around

; 表达式分组
; (group_expr) @block.around
; (group_expr
; content: (_) @block.inside)

; 字符串模板
; (string_template) @string.around
; (string_template
; interpolation: (_) @string.inside)
