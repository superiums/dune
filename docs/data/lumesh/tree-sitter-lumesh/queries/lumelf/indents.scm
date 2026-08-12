([
  (block)
  (group_expr)
  (group_expr content: (_))
  (params param: (_))
  (params param: (_))
  (function_call arg: (_))
  (module_call_expr arg: (_))
  (command_expr arg: (_))
  (command_expr redirect: (_))
  (chain_expr method: (_))
  (conditional_expr true_expr: (_))
  (conditional_expr false_expr: (_))
  (catch_expr handler: (_))
  (lambda_expr body: (_))
  (logical_and_expr right: (_))
  (logical_or_expr right: (_))
  (pipe_expr right: (_))
  (add_sub_expr right: (_))
  (mul_div_expr right: (_))
  (list element: (_))
  (map entry: (_))
  (namedmap entry: (_))
  (sets element: (_))
  ; "("
  "["
  "{"
  ; (if_expr then_branch: (_))h
  ; (if_expr else_branch: (_))
  (match_expr arm: (_))
] @indent @extend
(#set! "scope" "tail"))

([
  "}"
  "]"
  ; ")"
] @outdent
(#set! "scope" "all"))

((map_entry
    key: (_) @key
    value: (_) @val
    (#not-same-line? @key @val)
  ) @indent.always @extend
)

; Match return statements to prevent extending
(return_statement) @extend.prevent-once
