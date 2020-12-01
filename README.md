# Formal Languages and Automata Theory course practice
[![Build Status](https://img.shields.io/endpoint.svg?url=https%3A%2F%2Factions-badge.atrox.dev%2Flev-mxx%2Fflat-practice%2Fbadge&style=flat)](https://actions-badge.atrox.dev/lev-mxx/flat-practice/goto)

## Run tests
```make test```

## Request syntax
Example:
```
connect to path.db

define var1 as "a"*"b"?
define var2 as var1 | "c"

get edges from graph
get count of edges
    where (from, to, label) satify (from is start or to is final) and label is "label"
    from (graph g & var)
        with initials as [1, 2, 3] and finals as [6..90]  
```

`script := (_connect_expr_ | _define_expr_ | _get_expr_)*`

`connect_expt := connect to _ident_`

`define_expr := define _ident_ as _pattern_`

`get_expr := get _obj_expr_ from _graph_expr_`

`obj_expr := _list_expr_ | count of _list_expr_`

`list_expr := edges | _list_expr_ where (_ident_, _ident_, _ident_) satisfy _bool_expr_`

`bool_expr := _ident_ is final | _ident_ is start | _ident_ is string | _bool_expr_ _op_ _bool_expr_ | not _bool_expr_`

`graph_expr := ident | pattern | graph_expr & graph_expr | graph_expr with initials as set and finals as set`

`set := [_number_, ...] | [_number_.._number_]`

`pattern := (_pattern_elem_)*`

`pattern_elem := epsilon | _string_ | _ident_ | _pattern_elem_ '*' | _pattern_elem_ '?' | _pattern_elem_ '+' | '(' _pattern_elem_ ')'`
### Check syntax
`cargo run check [file]`
### Print AST in DOT format
`cargo run dot [file]`
