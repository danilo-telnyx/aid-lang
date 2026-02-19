use pest::Parser;
use pest_derive::Parser;

use crate::ast::*;

#[derive(Parser)]
#[grammar = "parser/aid.pest"]
pub struct AidParser;

/// Error type for parsing failures.
#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Parse error at {}:{}: {}", self.line, self.column, self.message)
    }
}

impl std::error::Error for ParseError {}

impl From<pest::error::Error<Rule>> for ParseError {
    fn from(e: pest::error::Error<Rule>) -> Self {
        let (line, column) = match e.line_col {
            pest::error::LineColLocation::Pos((l, c)) => (l, c),
            pest::error::LineColLocation::Span((l, c), _) => (l, c),
        };
        ParseError {
            message: e.to_string(),
            line,
            column,
        }
    }
}

type Pair<'a> = pest::iterators::Pair<'a, Rule>;
type Pairs<'a> = pest::iterators::Pairs<'a, Rule>;

fn span_from(pair: &Pair) -> Span {
    let (line, column) = pair.line_col();
    Span {
        line,
        column,
        offset: pair.as_span().start(),
    }
}

/// Parse AID source code into an AST Program.
pub fn parse_file(source: &str) -> Result<Program, ParseError> {
    let mut pairs = AidParser::parse(Rule::program, source)?;
    let program_pair = pairs.next().unwrap();
    parse_program(program_pair)
}

fn parse_program(pair: Pair) -> Result<Program, ParseError> {
    let span = span_from(&pair);
    let mut module = String::new();
    let mut imports = Vec::new();
    let mut declarations = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::module_decl => {
                module = parse_qualified_name(inner.into_inner().next().unwrap());
            }
            Rule::import_decl => {
                imports.push(parse_import(inner)?);
            }
            Rule::declaration => {
                let decl_inner = inner.into_inner().next().unwrap();
                declarations.push(parse_declaration(decl_inner)?);
            }
            Rule::EOI => {}
            _ => {}
        }
    }

    Ok(Program { module, imports, declarations, span })
}

fn parse_qualified_name(pair: Pair) -> String {
    pair.into_inner()
        .map(|p| p.as_str().to_string())
        .collect::<Vec<_>>()
        .join(".")
}

fn parse_qualified_name_parts(pair: Pair) -> Vec<String> {
    pair.into_inner()
        .map(|p| p.as_str().to_string())
        .collect()
}

fn parse_import(pair: Pair) -> Result<Import, ParseError> {
    let span = span_from(&pair);
    let mut inner = pair.into_inner();
    let qname_pair = inner.next().unwrap();
    let path = parse_qualified_name_parts(qname_pair);

    // Check for .{ items } or .*
    let kind = if let Some(next) = inner.next() {
        match next.as_rule() {
            Rule::ident_list => {
                let items: Vec<String> = next.into_inner().map(|p| p.as_str().to_string()).collect();
                ImportKind::Items(items)
            }
            _ => {
                // Must be glob (.*)
                ImportKind::Glob
            }
        }
    } else {
        // Simple module import — if it's a single name it could be Module or Item
        ImportKind::Module
    };

    Ok(Import { path, kind, span })
}

fn parse_declaration(pair: Pair) -> Result<Declaration, ParseError> {
    match pair.as_rule() {
        Rule::entity_decl => Ok(Declaration::Entity(parse_entity(pair)?)),
        Rule::function_decl => Ok(Declaration::Function(parse_function(pair)?)),
        Rule::reason_decl => Ok(Declaration::ReasonBlock(parse_reason(pair)?)),
        Rule::evolve_decl => Ok(Declaration::EvolveBlock(parse_evolve(pair)?)),
        Rule::contract_decl => Ok(Declaration::Contract(parse_contract(pair)?)),
        Rule::implement_decl => Ok(Declaration::Implement(parse_implement(pair)?)),
        Rule::const_decl => Ok(Declaration::Const(parse_const(pair)?)),
        Rule::type_alias => Ok(Declaration::TypeAlias(parse_type_alias(pair)?)),
        _ => Err(ParseError {
            message: format!("Unexpected declaration rule: {:?}", pair.as_rule()),
            line: 0,
            column: 0,
        }),
    }
}

// ── Entity ──────────────────────────────────────────────────────────────────

fn parse_entity(pair: Pair) -> Result<EntityDecl, ParseError> {
    let span = span_from(&pair);
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().to_string();
    let mut fields = Vec::new();
    let mut methods = Vec::new();

    for item in inner {
        match item.as_rule() {
            Rule::field_decl => fields.push(parse_field(item)?),
            Rule::method_decl => methods.push(parse_method(item)?),
            _ => {}
        }
    }

    Ok(EntityDecl { name, fields, methods, span })
}

fn parse_field(pair: Pair) -> Result<Field, ParseError> {
    let span = span_from(&pair);
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().to_string();
    let ty = parse_type_expr(inner.next().unwrap())?;
    let default = if let Some(expr_pair) = inner.next() {
        Some(parse_expression(expr_pair)?)
    } else {
        None
    };
    Ok(Field { name, ty, default, span })
}

fn parse_method(pair: Pair) -> Result<Function, ParseError> {
    let span = span_from(&pair);
    let mut inner = pair.into_inner();
    let mut is_private = false;

    // Check for "private" keyword
    let mut next = inner.next().unwrap();
    if next.as_str() == "private" {
        is_private = true;
        next = inner.next().unwrap();
    }

    let name = next.as_str().to_string();
    let params = parse_param_list(inner.next().unwrap())?;
    let return_type_pair = inner.next().unwrap();
    let return_type = Some(parse_type_expr(return_type_pair)?);

    let body_pair = inner.next().unwrap();
    let body = match body_pair.as_rule() {
        Rule::block => FunctionBody::Block(parse_block(body_pair)?),
        _ => FunctionBody::Expression(Box::new(parse_expression(body_pair)?)),
    };

    Ok(Function {
        name,
        params,
        return_type,
        body,
        is_async: false,
        is_private,
        span,
    })
}

// ── Function ────────────────────────────────────────────────────────────────

fn parse_function(pair: Pair) -> Result<Function, ParseError> {
    let span = span_from(&pair);
    let mut inner = pair.into_inner();
    let mut is_private = false;
    let mut is_async = false;

    let mut next = inner.next().unwrap();

    // Consume optional modifiers
    loop {
        match next.as_str() {
            "private" => {
                is_private = true;
                next = inner.next().unwrap();
            }
            "async" => {
                is_async = true;
                next = inner.next().unwrap();
            }
            _ => break,
        }
    }

    // next should be the ident (function name)
    let name = next.as_str().to_string();

    let param_pair = inner.next().unwrap();
    let params = parse_param_list(param_pair)?;

    let mut return_type = None;
    let mut body = FunctionBody::Block(vec![]);

    for remaining in inner {
        match remaining.as_rule() {
            Rule::return_type => {
                let rt_inner = remaining.into_inner().next().unwrap();
                match rt_inner.as_rule() {
                    Rule::tuple_type => {
                        let types: Result<Vec<AidType>, ParseError> = rt_inner
                            .into_inner()
                            .map(|p| parse_type_expr(p))
                            .collect();
                        return_type = Some(AidType::Tuple(types?));
                    }
                    _ => {
                        return_type = Some(parse_type_expr(rt_inner)?);
                    }
                }
            }
            Rule::block => {
                body = FunctionBody::Block(parse_block(remaining)?);
            }
            Rule::expression => {
                body = FunctionBody::Expression(Box::new(parse_expression(remaining)?));
            }
            _ => {}
        }
    }

    Ok(Function {
        name,
        params,
        return_type,
        body,
        is_async,
        is_private,
        span,
    })
}

fn parse_param_list(pair: Pair) -> Result<Vec<Param>, ParseError> {
    let mut params = Vec::new();
    for p in pair.into_inner() {
        if p.as_rule() == Rule::param {
            params.push(parse_param(p)?);
        }
    }
    Ok(params)
}

fn parse_param(pair: Pair) -> Result<Param, ParseError> {
    let span = span_from(&pair);
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().to_string();

    let mut ty = AidType::Inferred;
    let mut default = None;

    for remaining in inner {
        match remaining.as_rule() {
            Rule::type_expr => ty = parse_type_expr(remaining)?,
            Rule::expression => default = Some(parse_expression(remaining)?),
            _ => {}
        }
    }

    Ok(Param { name, ty, default, span })
}

// ── Type expressions ────────────────────────────────────────────────────────

fn parse_type_expr(pair: Pair) -> Result<AidType, ParseError> {
    let inner = pair.into_inner().next();
    let inner = match inner {
        Some(p) => p,
        None => return Ok(AidType::Inferred),
    };

    match inner.as_rule() {
        Rule::scalar_type => match inner.as_str() {
            "int" => Ok(AidType::Int),
            "float" => Ok(AidType::Float),
            "bool" => Ok(AidType::Bool),
            "string" => Ok(AidType::String),
            "byte" => Ok(AidType::Byte),
            _ => Ok(AidType::Inferred),
        },
        Rule::array_type => {
            let inner_type = parse_type_expr(inner.into_inner().next().unwrap())?;
            Ok(AidType::Array(Box::new(inner_type)))
        }
        Rule::map_type => {
            let mut parts = inner.into_inner();
            let key = parse_type_expr(parts.next().unwrap())?;
            let val = parse_type_expr(parts.next().unwrap())?;
            Ok(AidType::Map(Box::new(key), Box::new(val)))
        }
        Rule::option_type => {
            let inner_type = parse_type_expr(inner.into_inner().next().unwrap())?;
            Ok(AidType::Option(Box::new(inner_type)))
        }
        Rule::result_type => {
            let mut parts = inner.into_inner();
            let ok = parse_type_expr(parts.next().unwrap())?;
            let err = parse_type_expr(parts.next().unwrap())?;
            Ok(AidType::Result(Box::new(ok), Box::new(err)))
        }
        Rule::stream_type => {
            let inner_type = parse_type_expr(inner.into_inner().next().unwrap())?;
            Ok(AidType::Stream(Box::new(inner_type)))
        }
        Rule::fn_type => {
            let mut parts = inner.into_inner();
            let type_list = parts.next().unwrap();
            let param_types: Result<Vec<AidType>, ParseError> = type_list
                .into_inner()
                .map(|p| parse_type_expr(p))
                .collect();
            let ret = parse_type_expr(parts.next().unwrap())?;
            Ok(AidType::Fn(param_types?, Box::new(ret)))
        }
        Rule::ident => Ok(AidType::Entity(inner.as_str().to_string())),
        // type_expr itself can recurse
        Rule::type_expr => parse_type_expr(inner),
        _ => Ok(AidType::Inferred),
    }
}

// ── Block & Statements ──────────────────────────────────────────────────────

fn parse_block(pair: Pair) -> Result<Vec<Statement>, ParseError> {
    let mut stmts = Vec::new();
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::statement {
            stmts.push(parse_statement(inner)?);
        }
    }
    Ok(stmts)
}

fn parse_statement(pair: Pair) -> Result<Statement, ParseError> {
    let span = span_from(&pair);
    let inner = pair.into_inner().next().unwrap();

    match inner.as_rule() {
        Rule::return_stmt => {
            let expr = inner.into_inner().next()
                .map(|p| parse_expression(p))
                .transpose()?;
            Ok(Statement::Return { value: expr, span })
        }
        Rule::var_decl => {
            let mut parts = inner.into_inner();
            let mut mutable = false;
            let mut next = parts.next().unwrap();

            if next.as_str() == "mut" {
                mutable = true;
                next = parts.next().unwrap();
            }

            let name = next.as_str().to_string();

            // Could be `name := expr` or `name : type = expr`
            let next_part = parts.next().unwrap();
            let (ty, value) = match next_part.as_rule() {
                Rule::type_expr => {
                    let ty = parse_type_expr(next_part)?;
                    let val = parse_expression(parts.next().unwrap())?;
                    (Some(ty), val)
                }
                _ => {
                    // It's the expression directly (`:=` form)
                    (None, parse_expression(next_part)?)
                }
            };

            Ok(Statement::VarDecl { name, ty, value, mutable, span })
        }
        Rule::assignment => {
            let mut parts = inner.into_inner();
            let target_pair = parts.next().unwrap();
            let target = parse_lvalue(target_pair)?;
            let value = parse_expression(parts.next().unwrap())?;
            Ok(Statement::Assignment { target, value, span })
        }
        Rule::if_stmt => parse_if_stmt(inner),
        Rule::match_stmt => {
            let mut parts = inner.into_inner();
            let subject = parse_expression(parts.next().unwrap())?;
            let mut arms = Vec::new();
            for arm_pair in parts {
                if arm_pair.as_rule() == Rule::match_arm {
                    arms.push(parse_match_arm(arm_pair)?);
                }
            }
            Ok(Statement::Match { subject, arms, span })
        }
        Rule::for_stmt => {
            let stmt_span = span_from(&inner);
            let mut parts = inner.into_inner();
            let pattern_pair = parts.next().unwrap();
            let pattern = parse_for_pattern(pattern_pair)?;
            let iterable = parse_expression(parts.next().unwrap())?;
            let body = parse_block(parts.next().unwrap())?;
            Ok(Statement::For { pattern, iterable, body, span: stmt_span })
        }
        Rule::while_stmt => {
            let stmt_span = span_from(&inner);
            let mut parts = inner.into_inner();
            let condition = parse_expression(parts.next().unwrap())?;
            let body = parse_block(parts.next().unwrap())?;
            Ok(Statement::While { condition, body, span: stmt_span })
        }
        Rule::expression => {
            let expr = parse_expression(inner)?;
            Ok(Statement::Expression { expr, span })
        }
        _ => Err(ParseError {
            message: format!("Unexpected statement rule: {:?}", inner.as_rule()),
            line: span.line,
            column: span.column,
        }),
    }
}

fn parse_lvalue(pair: Pair) -> Result<Expression, ParseError> {
    let span = span_from(&pair);
    let mut parts = pair.into_inner();
    let name = parts.next().unwrap().as_str().to_string();
    let mut expr = Expression::Identifier { name, span: span.clone() };

    for access in parts {
        match access.as_rule() {
            Rule::member_access => {
                let member = access.into_inner().next().unwrap().as_str().to_string();
                expr = Expression::MemberAccess {
                    object: Box::new(expr),
                    member,
                    span: span.clone(),
                };
            }
            Rule::index_access => {
                let index = parse_expression(access.into_inner().next().unwrap())?;
                expr = Expression::Index {
                    object: Box::new(expr),
                    index: Box::new(index),
                    span: span.clone(),
                };
            }
            _ => {}
        }
    }

    Ok(expr)
}

fn parse_if_stmt(pair: Pair) -> Result<Statement, ParseError> {
    let span = span_from(&pair);
    let mut parts = pair.into_inner();
    let condition = parse_expression(parts.next().unwrap())?;
    let then_body = parse_block(parts.next().unwrap())?;

    let mut else_if_branches = Vec::new();
    let mut else_body = None;

    while let Some(next) = parts.next() {
        match next.as_rule() {
            Rule::if_stmt => {
                // else if — parse recursively then flatten
                let nested_span = span_from(&next);
                let mut nested_parts = next.into_inner();
                let cond = parse_expression(nested_parts.next().unwrap())?;
                let body = parse_block(nested_parts.next().unwrap())?;
                else_if_branches.push(ElseIfBranch {
                    condition: cond,
                    body,
                    span: nested_span,
                });
                // Continue consuming remaining else/else-if from nested
                for remaining in nested_parts {
                    match remaining.as_rule() {
                        Rule::if_stmt => {
                            let s = span_from(&remaining);
                            let mut rp = remaining.into_inner();
                            let c = parse_expression(rp.next().unwrap())?;
                            let b = parse_block(rp.next().unwrap())?;
                            else_if_branches.push(ElseIfBranch { condition: c, body: b, span: s });
                        }
                        Rule::block => {
                            else_body = Some(parse_block(remaining)?);
                        }
                        _ => {}
                    }
                }
            }
            Rule::block => {
                else_body = Some(parse_block(next)?);
            }
            _ => {}
        }
    }

    Ok(Statement::If {
        condition,
        then_body,
        else_if_branches,
        else_body,
        span,
    })
}

fn parse_for_pattern(pair: Pair) -> Result<Pattern, ParseError> {
    let mut parts: Vec<_> = pair.into_inner().collect();
    if parts.len() == 1 {
        Ok(Pattern::Identifier {
            name: parts.pop().unwrap().as_str().to_string(),
            binding: None,
        })
    } else if parts.len() == 2 {
        let names: Vec<String> = parts.iter().map(|p| p.as_str().to_string()).collect();
        Ok(Pattern::Destructure(names))
    } else {
        Ok(Pattern::Wildcard)
    }
}

fn parse_match_arm(pair: Pair) -> Result<MatchArm, ParseError> {
    let span = span_from(&pair);
    let mut parts = pair.into_inner();
    let pattern = parse_pattern(parts.next().unwrap())?;
    let body_pair = parts.next().unwrap();
    let body = match body_pair.as_rule() {
        Rule::block => MatchArmBody::Block(parse_block(body_pair)?),
        _ => MatchArmBody::Expression(parse_expression(body_pair)?),
    };
    Ok(MatchArm { pattern, body, span })
}

fn parse_pattern(pair: Pair) -> Result<Pattern, ParseError> {
    let singles: Vec<Pair> = pair.into_inner().collect();
    if singles.len() == 1 {
        parse_pattern_single(singles.into_iter().next().unwrap())
    } else {
        let mut patterns = Vec::new();
        for s in singles {
            patterns.push(parse_pattern_single(s)?);
        }
        Ok(Pattern::Multiple(patterns))
    }
}

fn parse_pattern_single(pair: Pair) -> Result<Pattern, ParseError> {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::wildcard_pattern => Ok(Pattern::Wildcard),
        Rule::range_pattern => {
            let mut parts = inner.into_inner();
            let start = parse_integer_literal(parts.next().unwrap());
            let end = parse_integer_literal(parts.next().unwrap());
            Ok(Pattern::Range {
                start: Box::new(start),
                end: Box::new(end),
            })
        }
        Rule::constructor_pattern => {
            let mut parts = inner.into_inner();
            let name = parts.next().unwrap().as_str().to_string();
            let binding = parts.next().map(|p| p.as_str().to_string());
            Ok(Pattern::Identifier { name, binding })
        }
        Rule::literal => {
            let lit = parse_literal_value(inner)?;
            Ok(Pattern::Literal(lit))
        }
        Rule::ident => Ok(Pattern::Identifier {
            name: inner.as_str().to_string(),
            binding: None,
        }),
        _ => Ok(Pattern::Wildcard),
    }
}

fn parse_integer_literal(pair: Pair) -> Expression {
    let span = span_from(&pair);
    let s = pair.as_str();
    let val = if s.starts_with("0x") || s.starts_with("0X") {
        i64::from_str_radix(&s[2..], 16).unwrap_or(0)
    } else {
        s.parse::<i64>().unwrap_or(0)
    };
    Expression::Literal { value: Literal::Int(val), span }
}

// ── Expressions ─────────────────────────────────────────────────────────────

fn parse_expression(pair: Pair) -> Result<Expression, ParseError> {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::try_expr => {
            let span = span_from(&inner);
            let expr = parse_expression(inner.into_inner().next().unwrap())?;
            Ok(Expression::Try { expr: Box::new(expr), span })
        }
        Rule::arrow_expr => parse_arrow_expr(inner),
        Rule::or_expr => parse_or_expr(inner),
        Rule::expression => parse_expression(inner),
        _ => parse_arrow_expr(inner),
    }
}

fn parse_arrow_expr(pair: Pair) -> Result<Expression, ParseError> {
    let span = span_from(&pair);
    let mut parts = pair.into_inner();
    let left = parse_or_expr(parts.next().unwrap())?;
    if let Some(right_pair) = parts.next() {
        let right = parse_or_expr(right_pair)?;
        Ok(Expression::BinaryOp {
            left: Box::new(left),
            op: BinaryOperator::Arrow,
            right: Box::new(right),
            span,
        })
    } else {
        Ok(left)
    }
}

fn parse_or_expr(pair: Pair) -> Result<Expression, ParseError> {
    let span = span_from(&pair);
    let mut parts = pair.into_inner();
    let mut left = parse_and_expr(parts.next().unwrap())?;
    while let Some(right_pair) = parts.next() {
        let right = parse_and_expr(right_pair)?;
        left = Expression::BinaryOp {
            left: Box::new(left),
            op: BinaryOperator::Or,
            right: Box::new(right),
            span: span.clone(),
        };
    }
    Ok(left)
}

fn parse_and_expr(pair: Pair) -> Result<Expression, ParseError> {
    let span = span_from(&pair);
    let mut parts = pair.into_inner();
    let mut left = parse_cmp_expr(parts.next().unwrap())?;
    while let Some(right_pair) = parts.next() {
        let right = parse_cmp_expr(right_pair)?;
        left = Expression::BinaryOp {
            left: Box::new(left),
            op: BinaryOperator::And,
            right: Box::new(right),
            span: span.clone(),
        };
    }
    Ok(left)
}

fn parse_cmp_expr(pair: Pair) -> Result<Expression, ParseError> {
    let span = span_from(&pair);
    let mut parts = pair.into_inner();
    let left = parse_range_expr(parts.next().unwrap())?;
    if let Some(op_pair) = parts.next() {
        let op = match op_pair.as_str() {
            "==" => BinaryOperator::Eq,
            "!=" => BinaryOperator::NotEq,
            "<=" => BinaryOperator::LtEq,
            ">=" => BinaryOperator::GtEq,
            "<" => BinaryOperator::Lt,
            ">" => BinaryOperator::Gt,
            _ => BinaryOperator::Eq,
        };
        let right = parse_range_expr(parts.next().unwrap())?;
        Ok(Expression::BinaryOp {
            left: Box::new(left),
            op,
            right: Box::new(right),
            span,
        })
    } else {
        Ok(left)
    }
}

fn parse_range_expr(pair: Pair) -> Result<Expression, ParseError> {
    let span = span_from(&pair);
    let mut parts = pair.into_inner();
    let left = parse_add_expr(parts.next().unwrap())?;
    if let Some(right_pair) = parts.next() {
        let right = parse_add_expr(right_pair)?;
        Ok(Expression::BinaryOp {
            left: Box::new(left),
            op: BinaryOperator::Range,
            right: Box::new(right),
            span,
        })
    } else {
        Ok(left)
    }
}

fn parse_add_expr(pair: Pair) -> Result<Expression, ParseError> {
    let span = span_from(&pair);
    let mut parts = pair.into_inner();
    let mut left = parse_mul_expr(parts.next().unwrap())?;
    while let Some(op_pair) = parts.next() {
        let op = match op_pair.as_str() {
            "+" => BinaryOperator::Add,
            "-" => BinaryOperator::Sub,
            _ => BinaryOperator::Add,
        };
        let right = parse_mul_expr(parts.next().unwrap())?;
        left = Expression::BinaryOp {
            left: Box::new(left),
            op,
            right: Box::new(right),
            span: span.clone(),
        };
    }
    Ok(left)
}

fn parse_mul_expr(pair: Pair) -> Result<Expression, ParseError> {
    let span = span_from(&pair);
    let mut parts = pair.into_inner();
    let mut left = parse_unary_expr(parts.next().unwrap())?;
    while let Some(op_pair) = parts.next() {
        let op = match op_pair.as_str() {
            "*" => BinaryOperator::Mul,
            "/" => BinaryOperator::Div,
            "%" => BinaryOperator::Mod,
            _ => BinaryOperator::Mul,
        };
        let right = parse_unary_expr(parts.next().unwrap())?;
        left = Expression::BinaryOp {
            left: Box::new(left),
            op,
            right: Box::new(right),
            span: span.clone(),
        };
    }
    Ok(left)
}

fn parse_unary_expr(pair: Pair) -> Result<Expression, ParseError> {
    let span = span_from(&pair);
    let mut parts: Vec<Pair> = pair.into_inner().collect();

    if parts.len() == 2 && parts[0].as_rule() == Rule::unary_op {
        let op = match parts[0].as_str() {
            "!" => UnaryOperator::Not,
            "-" => UnaryOperator::Neg,
            _ => UnaryOperator::Neg,
        };
        let operand = parse_unary_expr(parts.pop().unwrap())?;
        Ok(Expression::UnaryOp {
            op,
            operand: Box::new(operand),
            span,
        })
    } else {
        // await_expr
        parse_await_expr(parts.pop().unwrap())
    }
}

fn parse_await_expr(pair: Pair) -> Result<Expression, ParseError> {
    let span = span_from(&pair);
    let parts: Vec<Pair> = pair.into_inner().collect();

    // If there's an "await" keyword before the postfix_expr
    if parts.len() == 1 {
        parse_postfix_expr(parts.into_iter().next().unwrap())
    } else {
        // await keyword present — last element is the postfix_expr
        let expr = parse_postfix_expr(parts.into_iter().last().unwrap())?;
        Ok(Expression::Await { expr: Box::new(expr), span })
    }
}

fn parse_postfix_expr(pair: Pair) -> Result<Expression, ParseError> {
    let span = span_from(&pair);
    let mut parts = pair.into_inner();
    let mut expr = parse_primary_expr(parts.next().unwrap())?;

    for postfix in parts {
        match postfix.as_rule() {
            Rule::postfix_op => {
                let op_inner = postfix.into_inner().next().unwrap();
                match op_inner.as_rule() {
                    Rule::call_args => {
                        let args = parse_arg_list(op_inner.into_inner().next().unwrap())?;
                        expr = Expression::Call {
                            callee: Box::new(expr),
                            args,
                            span: span.clone(),
                        };
                    }
                    Rule::member_access => {
                        let member = op_inner.into_inner().next().unwrap().as_str().to_string();
                        expr = Expression::MemberAccess {
                            object: Box::new(expr),
                            member,
                            span: span.clone(),
                        };
                    }
                    Rule::index_access => {
                        let index = parse_expression(op_inner.into_inner().next().unwrap())?;
                        expr = Expression::Index {
                            object: Box::new(expr),
                            index: Box::new(index),
                            span: span.clone(),
                        };
                    }
                    _ => {}
                }
            }
            Rule::call_args => {
                let args = parse_arg_list(postfix.into_inner().next().unwrap())?;
                expr = Expression::Call {
                    callee: Box::new(expr),
                    args,
                    span: span.clone(),
                };
            }
            Rule::member_access => {
                let member = postfix.into_inner().next().unwrap().as_str().to_string();
                expr = Expression::MemberAccess {
                    object: Box::new(expr),
                    member,
                    span: span.clone(),
                };
            }
            Rule::index_access => {
                let index = parse_expression(postfix.into_inner().next().unwrap())?;
                expr = Expression::Index {
                    object: Box::new(expr),
                    index: Box::new(index),
                    span: span.clone(),
                };
            }
            _ => {}
        }
    }

    Ok(expr)
}

fn parse_arg_list(pair: Pair) -> Result<Vec<Argument>, ParseError> {
    let mut args = Vec::new();
    for arg_pair in pair.into_inner() {
        if arg_pair.as_rule() == Rule::argument {
            args.push(parse_argument(arg_pair)?);
        }
    }
    Ok(args)
}

fn parse_argument(pair: Pair) -> Result<Argument, ParseError> {
    let parts: Vec<Pair> = pair.into_inner().collect();
    if parts.len() == 2 && parts[0].as_rule() == Rule::ident {
        // Named argument: `name: expr`
        let name = parts[0].as_str().to_string();
        let value = parse_expression(parts.into_iter().nth(1).unwrap())?;
        Ok(Argument { name: Some(name), value })
    } else {
        // Positional argument
        let value = parse_expression(parts.into_iter().next().unwrap())?;
        Ok(Argument { name: None, value })
    }
}

fn parse_primary_expr(pair: Pair) -> Result<Expression, ParseError> {
    let span = span_from(&pair);
    let inner_pairs: Vec<Pair> = pair.into_inner().collect();

    // Tuple expression: (expr, expr, ...)
    if inner_pairs.len() >= 2 && inner_pairs.iter().all(|p| p.as_rule() == Rule::expression) {
        let elements: Result<Vec<Expression>, ParseError> = inner_pairs
            .into_iter()
            .map(|p| parse_expression(p))
            .collect();
        return Ok(Expression::Tuple { elements: elements?, span });
    }

    let inner = inner_pairs.into_iter().next().unwrap();

    match inner.as_rule() {
        Rule::expression => {
            // Grouped: (expr)
            parse_expression(inner)
        }
        Rule::lambda_expr => parse_lambda(inner),
        Rule::entity_instantiation => {
            let mut parts = inner.into_inner();
            let name = parts.next().unwrap().as_str().to_string();
            let field_list = parts.next().unwrap();
            let mut fields = Vec::new();
            for fi in field_list.into_inner() {
                if fi.as_rule() == Rule::field_init {
                    let mut fi_parts = fi.into_inner();
                    let fname = fi_parts.next().unwrap().as_str().to_string();
                    let fval = parse_expression(fi_parts.next().unwrap())?;
                    fields.push(FieldInit {
                        name: fname,
                        value: fval,
                        span: span.clone(),
                    });
                }
            }
            Ok(Expression::EntityInit { name, fields, span })
        }
        Rule::map_literal => {
            let mut entries = Vec::new();
            for entry in inner.into_inner() {
                if entry.as_rule() == Rule::map_entry {
                    let mut parts = entry.into_inner();
                    let key = parse_expression(parts.next().unwrap())?;
                    let val = parse_expression(parts.next().unwrap())?;
                    entries.push((key, val));
                }
            }
            Ok(Expression::MapLiteral { entries, span })
        }
        Rule::array_literal => {
            let elements: Result<Vec<Expression>, ParseError> = inner
                .into_inner()
                .next() // expr_list
                .map(|el| {
                    el.into_inner()
                        .map(|p| parse_expression(p))
                        .collect()
                })
                .unwrap_or(Ok(vec![]));
            Ok(Expression::ArrayLiteral { elements: elements?, span })
        }
        Rule::some_expr => {
            let expr = parse_expression(inner.into_inner().next().unwrap())?;
            Ok(Expression::Wrap {
                kind: WrapKind::Some,
                value: Box::new(expr),
                span,
            })
        }
        Rule::none_literal => Ok(Expression::Literal { value: Literal::None, span }),
        Rule::ok_expr => {
            let expr = parse_expression(inner.into_inner().next().unwrap())?;
            Ok(Expression::Wrap {
                kind: WrapKind::Ok,
                value: Box::new(expr),
                span,
            })
        }
        Rule::err_expr => {
            let expr = parse_expression(inner.into_inner().next().unwrap())?;
            Ok(Expression::Wrap {
                kind: WrapKind::Err,
                value: Box::new(expr),
                span,
            })
        }
        Rule::literal => {
            let lit = parse_literal_value(inner)?;
            Ok(Expression::Literal { value: lit, span })
        }
        Rule::ident => {
            let name = inner.as_str().to_string();
            if name == "break" {
                Ok(Expression::Identifier { name: "break".to_string(), span })
            } else if name == "continue" {
                Ok(Expression::Identifier { name: "continue".to_string(), span })
            } else {
                Ok(Expression::Identifier { name, span })
            }
        }
        _ => Ok(Expression::Identifier {
            name: inner.as_str().to_string(),
            span,
        }),
    }
}

fn parse_lambda(pair: Pair) -> Result<Expression, ParseError> {
    let span = span_from(&pair);
    let mut parts = pair.into_inner();
    let params = parse_param_list(parts.next().unwrap())?;
    let mut return_type = None;
    let mut body = FunctionBody::Block(vec![]);

    for remaining in parts {
        match remaining.as_rule() {
            Rule::type_expr => {
                return_type = Some(parse_type_expr(remaining)?);
            }
            Rule::block => {
                body = FunctionBody::Block(parse_block(remaining)?);
            }
            Rule::expression => {
                body = FunctionBody::Expression(Box::new(parse_expression(remaining)?));
            }
            _ => {}
        }
    }

    Ok(Expression::Lambda { params, return_type, body, span })
}

fn parse_literal_value(pair: Pair) -> Result<Literal, ParseError> {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::integer_literal => {
            let s = inner.as_str();
            let val = if s.starts_with("0x") || s.starts_with("0X") {
                i64::from_str_radix(&s[2..], 16).unwrap_or(0)
            } else {
                s.parse::<i64>().unwrap_or(0)
            };
            Ok(Literal::Int(val))
        }
        Rule::float_literal => {
            let val = inner.as_str().parse::<f64>().unwrap_or(0.0);
            Ok(Literal::Float(val))
        }
        Rule::bool_literal => {
            Ok(Literal::Bool(inner.as_str() == "true"))
        }
        Rule::string_literal => {
            let raw = inner.as_str();
            // Strip surrounding quotes
            let content = &raw[1..raw.len() - 1];
            // Unescape basic sequences
            let unescaped = content
                .replace("\\n", "\n")
                .replace("\\t", "\t")
                .replace("\\r", "\r")
                .replace("\\\\", "\\")
                .replace("\\\"", "\"")
                .replace("\\0", "\0");
            Ok(Literal::String(unescaped))
        }
        _ => Ok(Literal::None),
    }
}

// ── Reason ──────────────────────────────────────────────────────────────────

fn parse_reason(pair: Pair) -> Result<ReasonBlock, ParseError> {
    let span = span_from(&pair);
    let mut inner = pair.into_inner();
    let mut mode = ReasonMode::Static;

    let mut next = inner.next().unwrap();
    if next.as_rule() == Rule::reason_mode_annotation {
        let mode_str = next.into_inner().next().unwrap().as_str();
        mode = match mode_str {
            "dynamic" => ReasonMode::Dynamic,
            _ => ReasonMode::Static,
        };
        next = inner.next().unwrap();
    }

    let name = next.as_str().to_string();
    let params = parse_param_list(inner.next().unwrap())?;
    let return_type = parse_type_expr(inner.next().unwrap())?;

    let body_pair = inner.next().unwrap();
    let mut goal = String::new();
    let mut constraints = Vec::new();
    let mut examples = Vec::new();
    let mut context = Vec::new();
    let mut fallback = None;

    for part in body_pair.into_inner() {
        match part.as_rule() {
            Rule::reason_goal => {
                let s = part.into_inner().next().unwrap().as_str();
                goal = s[1..s.len() - 1].to_string(); // strip quotes
            }
            Rule::reason_constraints => {
                for sl in part.into_inner() {
                    if sl.as_rule() == Rule::string_list {
                        for s in sl.into_inner() {
                            let raw = s.as_str();
                            constraints.push(raw[1..raw.len() - 1].to_string());
                        }
                    }
                }
            }
            Rule::reason_context => {
                for el in part.into_inner() {
                    if el.as_rule() == Rule::expr_list {
                        for e in el.into_inner() {
                            if let Ok(expr) = parse_expression(e) {
                                context.push(expr);
                            }
                        }
                    }
                }
            }
            Rule::reason_examples => {
                for el in part.into_inner() {
                    if el.as_rule() == Rule::example_list {
                        for entry in el.into_inner() {
                            if entry.as_rule() == Rule::example_entry {
                                let mut ep = entry.into_inner();
                                let input = parse_expression(ep.next().unwrap())?;
                                let output = parse_expression(ep.next().unwrap())?;
                                examples.push(ReasonExample { input, output });
                            }
                        }
                    }
                }
            }
            Rule::reason_fallback => {
                fallback = Some(parse_expression(part.into_inner().next().unwrap())?);
            }
            _ => {}
        }
    }

    Ok(ReasonBlock {
        name, mode, params, return_type, goal, constraints, examples, context, fallback, span,
    })
}

// ── Evolve ──────────────────────────────────────────────────────────────────

fn parse_evolve(pair: Pair) -> Result<EvolveBlock, ParseError> {
    let span = span_from(&pair);
    let mut inner = pair.into_inner();
    let target = inner.next().unwrap().as_str().to_string();

    let mut track = false;
    let mut retrain_every = None;
    let mut min_accuracy = None;
    let mut storage = None;
    let mut approve = None;

    for field in inner {
        if field.as_rule() == Rule::evolve_field {
            let mut fp = field.into_inner();
            let key = fp.next().unwrap().as_str();
            let val_pair = fp.next().unwrap();
            match key {
                "track" => {
                    if let Ok(Expression::Literal { value: Literal::Bool(b), .. }) = parse_expression(val_pair) {
                        track = b;
                    }
                }
                "retrain_every" | "retrainEvery" => {
                    if let Ok(Expression::Literal { value: Literal::Int(n), .. }) = parse_expression(val_pair) {
                        retrain_every = Some(n);
                    }
                }
                "min_accuracy" | "minAccuracy" => {
                    if let Ok(Expression::Literal { value: Literal::Float(f), .. }) = parse_expression(val_pair) {
                        min_accuracy = Some(f);
                    }
                }
                "storage" => {
                    if let Ok(Expression::Literal { value: Literal::String(s), .. }) = parse_expression(val_pair) {
                        storage = Some(s);
                    }
                }
                "approve" => {
                    if let Ok(Expression::Literal { value: Literal::Bool(b), .. }) = parse_expression(val_pair) {
                        approve = Some(b);
                    }
                }
                _ => {}
            }
        }
    }

    Ok(EvolveBlock { target, track, retrain_every, min_accuracy, storage, approve, span })
}

// ── Contract ────────────────────────────────────────────────────────────────

fn parse_contract(pair: Pair) -> Result<Contract, ParseError> {
    let span = span_from(&pair);
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().to_string();
    let mut rules = Vec::new();
    let mut methods = Vec::new();

    for item in inner {
        match item.as_rule() {
            Rule::contract_rule => {
                let raw = item.into_inner().next().unwrap().as_str();
                rules.push(raw[1..raw.len() - 1].to_string());
            }
            Rule::fn_signature => {
                let sig_span = span_from(&item);
                let mut sp = item.into_inner();
                let fname = sp.next().unwrap().as_str().to_string();
                let params = parse_param_list(sp.next().unwrap())?;
                let ret = parse_type_expr(sp.next().unwrap())?;
                methods.push(FnSignature {
                    name: fname,
                    params,
                    return_type: ret,
                    span: sig_span,
                });
            }
            _ => {}
        }
    }

    Ok(Contract { name, rules, methods, span })
}

// ── Implement ───────────────────────────────────────────────────────────────

fn parse_implement(pair: Pair) -> Result<ImplementBlock, ParseError> {
    let span = span_from(&pair);
    let mut inner = pair.into_inner();
    let contract_name = inner.next().unwrap().as_str().to_string();
    let mut methods = Vec::new();

    for item in inner {
        if item.as_rule() == Rule::function_decl {
            methods.push(parse_function(item)?);
        }
    }

    Ok(ImplementBlock { contract_name, methods, span })
}

// ── Const & TypeAlias ───────────────────────────────────────────────────────

fn parse_const(pair: Pair) -> Result<ConstDecl, ParseError> {
    let span = span_from(&pair);
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().to_string();
    let value = parse_expression(inner.next().unwrap())?;
    Ok(ConstDecl { name, value, span })
}

fn parse_type_alias(pair: Pair) -> Result<TypeAliasDecl, ParseError> {
    let span = span_from(&pair);
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().to_string();
    let ty = parse_type_expr(inner.next().unwrap())?;
    Ok(TypeAliasDecl { name, ty, span })
}
