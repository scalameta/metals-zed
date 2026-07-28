(_ "{" @start "}" @end) @indent
(_ "(" @start ")" @end) @indent
(_ "[" @start "]" @end) @indent

(template_body ":" @start) @indent
(enum_body ":" @start) @indent

(val_definition
  "=" @start
  value: [
    (indented_block)
    (indented_cases)
  ] @indent)

(var_definition
  "=" @start
  value: [
    (indented_block)
    (indented_cases)
  ] @indent)

(function_definition
  "=" @start
  body: [
    (indented_block)
    (indented_cases)
  ] @indent)

(given_definition
  "=" @start
  body: [
    (indented_block)
    (indented_cases)
  ] @indent)

(given_definition
  ["with" ":"] @start
  .
  body: (with_template_body) @indent)

(extension_definition
  parameters: (parameters) @start) @indent

(if_expression
  "if" @start
  condition: (indented_block) @indent)

(if_expression
  "then" @start
  consequence: [
    (indented_block)
    (indented_cases)
  ] @indent)

(if_expression
  "else" @start
  alternative: [
    (indented_block)
    (indented_cases)
  ] @indent)

(if_expression
  condition: (parenthesized_expression) @start
  consequence: (indented_block) @indent)

(while_expression
  "while" @start
  condition: (indented_block) @indent)

(while_expression
  "do" @start
  body: [
    (indented_block)
    (indented_cases)
  ] @indent)

(while_expression
  condition: (parenthesized_expression) @start
  body: (_) @indent)

(do_while_expression
  "do" @start
  body: (_) @indent)

(for_expression
  "for" @start
  enumerators: (enumerators) @indent)

(for_expression
  ["do" "yield"] @start
  body: (_) @indent)

(for_expression
  [")" "}"] @start
  body: (_) @indent)

(match_expression
  "match" @start
  body: (indented_cases) @indent)

(try_expression
  "try" @start
  body: (_) @indent)

(catch_clause
  "catch" @start) @indent

(finally_clause
  "finally" @start) @indent

(case_clause
  "=>" @start) @indent

(lambda_expression
  parameters: (_)
  ["=>" "?=>"] @start) @indent

(call_expression
  ":" @start
  arguments: (colon_argument) @indent)

(match_type
  "match" @start) @indent

(type_case_clause
  "=>" @start) @indent

[
  (type_definition "=" @start)
  (assignment_expression "=" @start)
  (parameter "=" @start)
  (class_parameter "=" @start)
] @indent