// use std::fmt::{Debug, Formatter, self};
use std::rc::Rc;

use crate::tokenizer::{Token};
use crate::tokenizer::TType::{self, String, Number, Name, Op as Operator, NL, EndMarker, Newline};

use peg::str::LineCol;
use peg::{parser, Parse, ParseElem, RuleResult};


#[derive(Debug)]
pub struct TokVec(Vec<Rc<Token>>);

impl std::convert::From<Vec<Token>> for TokVec {
    fn from(vec: Vec<Token>) -> Self {
        TokVec(vec.into_iter().map(Rc::new).collect())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseLoc {
    pub start_pos: LineCol,
    pub end_pos: LineCol,
}

impl std::fmt::Display for ParseLoc {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.start_pos.fmt(f)
    }
}


impl Parse for TokVec {
    type PositionRepr = ParseLoc;

    fn start<'input>(&'input self) -> usize {
        0
    }

    fn is_eof<'input>(&'input self, p: usize) -> bool {
        p >= self.0.len()
    }

    fn position_repr<'input>(&'input self, p: usize) -> Self::PositionRepr {
        let tok = self.0.get(p).unwrap_or_else(||self.0.last().unwrap());
        ParseLoc {
            start_pos: LineCol {
                line: tok.start.line,
                column: tok.start.col,
                offset: tok.start.col,
            },
            end_pos: LineCol {
                line: tok.end.line,
                column: tok.end.col,
                offset: tok.end.col,
            },
        }
    }
}

type TokenRef = Rc<Token>;

impl ParseElem for TokVec {
    type Element = TokenRef;

    fn parse_elem(&self, pos: usize) -> RuleResult<Self::Element> {
        match self.0.get(pos) {
            Some(tok) => RuleResult::Matched(pos+1, tok.clone()),
            None => RuleResult::Failed,
        }
    }
}

pub struct ValueNode {
    pub result: u32,
}

// impl std::Debug for ValueNode {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("ValueNode")
//             .field("result", &self.result)
//             .finish()
//     }
//
// }

parser! {
    pub grammar python() for TokVec {


        //Starting rules

        pub rule file(name: &str) -> Module
        = traced(<_file(name)>)

        pub rule expression_input() -> Expression
        = traced(<e:star_expressions() tok(NL, "Newline") tok(EndMarker, "EOF") {  make_expression(e)  }> )

        pub rule statement_input() -> Statement
        = traced(<s:statement() tok(EndMarker, "EOF") { make_interactive(s) }>)

        pub rule _file(name: &str) -> Module
        = s:statements()? eof:tok(EndMarker, "EOF") {
                make_module(name, s.unwrap_or_default(), eof)
        }

        // pub rule fstring() -> FString
        // = traced(<e:star_expressions()>) { make_fstring(e) }


        //General statements

        rule statements() -> Vec<Statement>
        = statement()+

        rule statement() -> Statement
        = c:compound_stmt() { Statement::Compound(c) }
        / s:simple_stmt() {
            Statement::Simple(make_simple_statement_lines(s))
        }

        rule simple_stmts() -> SimpleStatementParts
        = first_tok:&_ stmts:separated_trailer(<simple_stmt()>, <lit(";")>) nl:tok(NL, "NEWLINE") {
            SimpleStatementParts {
                first_tok,
                first_statement: stmts.0,
                rest: stmts.1,
                last_semi: stmts.2,
                nl
            }
        }

        #[cache]
        rule simple_stmt() -> SmallStatement
        = assignment()
        / e:star_expressions() { SmallStatement::Expr(Expr { value: e, semicolon: None }) }
            / &lit("return") s:return_stmt() { SmallStatement::Return(s) }
            // this is expanded from the original grammar's import_stmt rule
            / &lit("import") i:import_name() { SmallStatement::Import(i) }
            / &lit("from") i:import_from() { SmallStatement::ImportFrom(i) }
            / &lit("raise") r:raise_stmt() { SmallStatement::Raise(r) }
            / lit("pass") { SmallStatement::Pass(Pass { semicolon: None }) }
            / &lit("del") s:del_stmt() { SmallStatement::Del(s) }
            / &lit("yield") s:yield_stmt() { SmallStatement::Expr(Expr { value: s, semicolon: None }) }
            / &lit("assert") s:assert_stmt() {SmallStatement::Assert(s)}
            / lit("break") { SmallStatement::Break(Break { semicolon: None })}
            / lit("continue") { SmallStatement::Continue(Continue { semicolon: None })}
            / &lit("global") s:global_stmt() {SmallStatement::Global(s)}
            / &lit("nonlocal") s:nonlocal_stmt() {SmallStatement::Nonlocal(s)}



        rule compound_stmt() -> CompoundStatement
            = &(lit("def") / lit("@") / tok(Async, "ASYNC")) f:function_def() {
                CompoundStatement::FunctionDef(f)
            }
            / &lit("if") f:if_stmt() { CompoundStatement::If(f) }
            / &(lit("class") / lit("@")) c:class_def() { CompoundStatement::ClassDef(c) }
            / &(lit("with") / tok(Async, "ASYNC")) w:with_stmt() { CompoundStatement::With(w) }
            / &(lit("for") / tok(Async, "ASYNC")) f:for_stmt() { CompoundStatement::For(f) }
            / &lit("try") t:try_stmt() { CompoundStatement::Try(t) }
            / &lit("try") t:try_star_stmt() { CompoundStatement::TryStar(t) }
            / &lit("while") w:while_stmt() { CompoundStatement::While(w) }
            / m:match_stmt() { CompoundStatement::Match(m) }


        // "Simple" statemens
        // TODO: there's an extra '(' single_target ')' clause here in upstream
        // I don't remember this  syntax and will have to hunt it down
        // a=(lit("(") b=single_target lit(")")) { b }

        //         / single_subscript_attribute_target) ':' b=expression c=['=' d=annotated_rhs { d }] {
        // CHECK_VERSION(stmt_ty, 6, "Variable annotations syntax is", _PyAST_AnnAssign(a, b, c, 0, EXTRA)) }

        rule assignment() -> SmallStatement
            = a:name() col:lit(":") ann:expression()
                rhs:(eq:lit("=") d:annotated_rhs() {(eq, d)})? {
                    SmallStatement::AnnAssign(make_ann_assignment(
                        AssignTargetExpression::Name(Box::new(a)), col, ann, rhs))
            }
            / a:single_subscript_attribute_target() col:lit(":") ann:expression()
                rhs:(eq:lit("=") d:annotated_rhs() {(eq, d)})? {
                    SmallStatement::AnnAssign(make_ann_assignment(a, col, ann, rhs))
            }
            / lhs:(t:star_targets() eq:lit("=") {(t, eq)})+ rhs:(yield_expr() / star_expressions()) !lit("=") {
                SmallStatement::Assign(make_assignment(lhs, rhs))
            }
            / t:single_target() op:augassign() rhs:(yield_expr() / star_expressions()) {
                SmallStatement::AugAssign(make_aug_assign(t, op, rhs))
            }

        rule annotated_rhs() -> Expression
            = yield_expr() / star_expressions()


        //Deviates from libcst
        rule augassign() -> AugOp
            = &(lit("+=") { AugOp::PlusEq }
                / lit("-=") { AugOp::MinusEq }
                / lit("*=") { AugOp::MulEq }
                / lit("@=") { AugOp::MatrixEq }
                /  lit("/=") { AugOp::DivEq }
                / lit("%=")  { AugOp::ModulaEq }
                / lit("&=")  { AugOp::AndEq }
                / lit("|=") { AugOp::OrEq }
                / lit("^=") { AugOp::NotEq }
                / lit("<<=") { AugOp::LeftShitEq }
                / lit(">>=") { AugOp::RightShiftEq }
                / lit("**=") { AugOp::ExponentEq }
                / lit("//=")) { AugO::FloorDivEq }


        rule return_stmt() -> Return
            = kw:lit("return") a:star_expressions()?
        { make_return(kw, a) }


        rule raise_stmt() -> Raise
            = kw:lit("raise") exc:expression()
                rest:(f:lit("from")cause:expression() {(f, cau)})?
        {   make_raise(kw, Some(exc), rest)  }
        / kw:lit("Raise") { make_raise(kw, None, Nobne ) }

        //TODO play around with this, the greedy star seems like a weird spot to me
        rule global_stmt() -> Global
            = kw:lit("global") init:(n:name() c:comma() {(n, c) })* last:name() {
            make_global(kw, init, last)
        }

        rule nonlocal_stmt() -> Nonlocal
        = kw:lit("nonlocal") init:(n:name() c:comma() {(n, c)})* last:name() {
                make_nonlocal(kw, init, last)
        }

        rule del_stmt() -> Del
            = kw:lit("del") t:del_target() &(lit(";") / tok(NL, "NEWLINE")) {
                make_del(kw, t)
            }
            / kw:lit("del") t:del_targets() &(lit(";") / tok(NL, "NEWLINE")) {
                make_del(kw, make_del_tuple(None, t, None))
            }

        rule yield_stmt() -> Expression
            = yield_expr()

        rule assert_stmt() -> Assert
            = kw:lit("assert") test:expression() rest:(c:comma() msg:expression() {(c, msg)})? {
                make_assert(kw, test, rest)
            }

        rule import_name() -> Import
            = kw:lit("import") a:dotted_as_names() {
                make_import(kw, a)
            }

        rule import_from() -> ImportFrom
            = from:lit("from") dots:dots()? m:dotted_name()
                import:lit("import") als:import_from_targets() {
                    make_import_from(from, dots.unwrap_or_default(), Some(m), import, als)
            }
            / from:lit("from") dots:dots()
                import:lit("import") als:import_from_targets() {
                    make_import_from(from, dots, None, import, als)
            }

        rule import_from_targets() -> ParenthesizedImportNames
            = lpar:lpar() als:import_from_as_names() c:comma()? rpar:rpar() {
                let mut als = als;
                if let (comma@Some(_), Some(mut last)) = (c, als.last_mut()) {
                    last.comma = comma;
                }
                (Some(lpar), ImportNames::Aliases(als), Some(rpar))
            }
            / als:import_from_as_names() !lit(",") { (None, ImportNames::Aliases(als), None)}
            / star:lit("*") { (None, ImportNames::Star(ImportStar {}), None) }

        rule import_from_as_names() -> Vec<ImportAlias>
            = items:separated(<import_from_as_name()>, <comma()>) {
                make_import_from_as_names(items.0, items.1)
            }

        rule import_from_as_name() -> ImportAlias
            = n:name() asname:(kw:lit("as") z:name() {(kw, z)})? {
                make_import_alias(NameOrAttribute::N(Box::new(n)), asname)
            }

        rule dotted_as_names() -> Vec<ImportAlias>
            = init:(d:dotted_as_name() c:comma() {d.with_comma(c)})*
                last:dotted_as_name() {
                    concat(init, vec![last])
            }

        rule dotted_as_name() -> ImportAlias
            = n:dotted_name() asname:(kw:lit("as") z:name() {(kw, z)})? {
                make_import_alias(n, asname)
            }

        // TODO: libcst asks why does this diverge from CPython?
        rule dotted_name() -> NameOrAttribute
            = first:name() tail:(dot:lit(".") n:name() {(dot, n)})* {
                make_name_or_attr(first, tail)
            }

        //1. Compound statements

        // 1.a Common elements

        // Common elements

        #[cache]
        rule block() -> Suite
            = n:tok(NL, "NEWLINE") ind:tok(Indent, "INDENT") s:statements() ded:tok(Dedent, "DEDENT") {
                make_indented_block(n, ind, s, ded)
            }
            / s:simple_stmts() {
                make_simple_statement_suite(s)
            }

        rule decorators() -> Vec<Decorator>
            = (at:lit("@") e:named_expression() nl:tok(NL, "NEWLINE") {
                make_decorator(at, e, nl)
            } )+

        // Class definitions

        rule class_def() -> ClassDef
            = d:decorators() c:class_def_raw() { c.with_decorators(d) }
            / class_def_raw()

        rule class_def_raw() -> ClassDef
            = kw:lit("class") n:name() arg:(l:lpar() a:arguments()? r:rpar() {(l, a, r)})?
                col:lit(":") b:block() {?
                    make_class_def(kw, n, arg, col, b)
            }

        // Function definitions

        rule function_def() -> FunctionDef
            = d:decorators() f:function_def_raw() {f.with_decorators(d)}
            / function_def_raw()

        rule _returns() -> Annotation
            = l:lit("->") e:expression() {
                make_annotation(l, e)
            }

        rule function_def_raw() -> FunctionDef
            = def:lit("def") n:name() op:lit("(") params:params()?
                cp:lit(")") ty:_returns()? c:lit(":") b:block() {
                    make_function_def(None, def, n, op, params, cp, ty, c, b)
            }
            / asy:tok(Async, "ASYNC") def:lit("def") n:name() op:lit("(") params:params()?
                cp:lit(")") ty:_returns()? c:lit(":") b:block() {
                    make_function_def(Some(asy), def, n, op, params, cp, ty, c, b)
            }

        // Function parameters

        rule params() -> Parameters
            = parameters()

        rule parameters() -> Parameters
            = a:slash_no_default() b:param_no_default()* c:param_with_default()*  d:star_etc()? {
                make_parameters(Some(a), concat(b, c), d)
            }
            / a:slash_with_default() b:param_with_default()* d:star_etc()? {
                make_parameters(Some(a), b, d)
            }
            / a:param_no_default()+ b:param_with_default()* d:star_etc()? {
                make_parameters(None, concat(a, b), d)
            }
            / a:param_with_default()+ d:star_etc()? {
                make_parameters(None, a, d)
            }
            / d:star_etc() {
                make_parameters(None, vec![], Some(d))
            }

        rule slash_no_default() -> (Vec<Param>, ParamSlash)
            = a:param_no_default()+ slash:lit("/") com:comma() {
                    (a, ParamSlash { comma: Some(com)})
            }
            / a:param_no_default()+ slash:lit("/") &lit(")") {
                (a, ParamSlash { comma: None })
            }

        rule slash_with_default() -> (Vec<Param>, ParamSlash)
            = a:param_no_default()* b:param_with_default()+ slash:lit("/") c:comma() {
                (concat(a, b), ParamSlash { comma: Some(c) })
            }
            / a:param_no_default()* b:param_with_default()+ slash:lit("/") &lit(")") {
                (concat(a, b), ParamSlash { comma: None })
            }

        rule star_etc() -> StarEtc
            = star:lit("*") a:param_no_default() b:param_maybe_default()* kw:kwds()? {
                StarEtc(Some(StarArg::Param(Box::new(
                    add_param_star(a, star)))), b, kw)
            }
            / lit("*") c:comma() b:param_maybe_default()+ kw:kwds()? {
                StarEtc(Some(StarArg::Star(Box::new(ParamStar {comma:c }))), b, kw)
            }
            / kw:kwds() { StarEtc(None, vec![], Some(kw)) }

        rule kwds() -> Param
            = star:lit("**") a:param_no_default() {
                add_param_star(a, star)
            }

        rule param_no_default() -> Param
            = a:param() c:lit(",") { add_param_default(a, None, Some(c)) }
            / a:param() &lit(")") {a}

        rule param_with_default() -> Param
            = a:param() def:default() c:lit(",") {
                add_param_default(a, Some(def), Some(c))
            }
            / a:param() def:default() &lit(")") {
                add_param_default(a, Some(def), None)
            }

        rule param_maybe_default() -> Param
            = a:param() def:default()? c:lit(",") {
                add_param_default(a, def, Some(c))
            }
            / a:param() def:default()? &lit(")") {
                add_param_default(a, def, None)
            }

        rule param() -> Param
            = n:name() a:annotation()? {
                Param {name: n, annotation: a, ..Default::default() }
            }

        rule annotation() -> Annotation
            = col:lit(":") e:expression() {
                make_annotation(col, e)
            }

        rule default() -> (AssignEqual, Expression)
            = eq:lit("=") ex:expression() {
                (make_assign_equal(eq), ex)
            }

        // If statement

        rule if_stmt() -> If
            = i:lit("if") a:named_expression() col:lit(":") b:block() elif:elif_stmt() {
                make_if(i, a, col, b, Some(OrElse::Elif(elif)), false)
            }
            / i:lit("if") a:named_expression() col:lit(":") b:block() el:else_block()? {
                make_if(i, a, col, b, el.map(OrElse::Else), false)
            }

        rule elif_stmt() -> If
            = i:lit("elif") a:named_expression() col:lit(":") b:block() elif:elif_stmt() {
                make_if(i, a, col, b, Some(OrElse::Elif(elif)), true)
            }
            / i:lit("elif") a:named_expression() col:lit(":") b:block() el:else_block()? {
                make_if(i, a, col, b, el.map(OrElse::Else), true)
            }

        rule else_block() -> Else
            = el:lit("else") col:lit(":") b:block() {
                make_else(el, col, b)
            }

        // While statement

        rule while_stmt() -> While
            = kw:lit("while") test:named_expression() col:lit(":") b:block() el:else_block()? {
                make_while(kw, test, col, b, el)
            }

        // For statement

        rule for_stmt() -> For
            = f:lit("for") t:star_targets() i:lit("in") it:star_expressions()
                c:lit(":") b:block() el:else_block()? {
                    make_for(None, f, t, i, it, c, b, el)
            }
            / asy:tok(Async, "ASYNC") f:lit("for") t:star_targets() i:lit("in")
                it:star_expressions()
                c:lit(":") b:block() el:else_block()? {
                    make_for(Some(asy), f, t, i, it, c, b, el)
            }

        // With statement

        rule with_stmt() -> With
            = kw:lit("with") l:lpar() items:separated_trailer(<with_item()>, <comma()>) r:rpar()
                col:lit(":") b:block() {
                    make_with(None, kw, Some(l), comma_separate(items.0, items.1, items.2), Some(r), col, b)
            }
            / kw:lit("with") items:separated(<with_item()>, <comma()>)
                col:lit(":") b:block() {
                    make_with(None, kw, None, comma_separate(items.0, items.1, None), None, col, b)
            }
            / asy:tok(Async, "ASYNC") kw:lit("with") l:lpar() items:separated_trailer(<with_item()>, <comma()>) r:rpar()
                col:lit(":") b:block() {
                    make_with(Some(asy), kw, Some(l), comma_separate(items.0, items.1, items.2), Some(r), col, b)
            }
            / asy:tok(Async, "ASYNC") kw:lit("with") items:separated(<with_item()>, <comma()>)
                col:lit(":") b:block() {
                    make_with(Some(asy), kw, None, comma_separate(items.0, items.1, None), None, col, b)
            }

        rule with_item() -> WithItem
            = e:expression() a:lit("as") t:star_target() &(lit(",") / lit(":")) {
                make_with_item(e, Some(a), Some(t))
            }
            / e:expression() {
                make_with_item(e, None, None)
            }

        // Try statement

        rule try_stmt() -> Try
            = kw:lit("try") lit(":") b:block() f:finally_block() {
                make_try(kw, b, vec![], None, Some(f))
            }
            / kw:lit("try") lit(":") b:block() ex:except_block()+ el:else_block()?
                f:finally_block()? {
                    make_try(kw, b, ex, el, f)
            }

        // Note: this is separate because TryStar is a different type in LibCST
        rule try_star_stmt() -> TryStar
            = kw:lit("try") lit(":") b:block() ex:except_star_block()+
                el:else_block()? f:finally_block()? {
                    make_try_star(kw, b, ex, el, f)
            }

        // Except statement

        rule except_block() -> ExceptHandler
            = kw:lit("except") e:expression() a:(k:lit("as") n:name() {(k, n)})?
                col:lit(":") b:block() {
                    make_except(kw, Some(e), a, col, b)
            }
            / kw:lit("except") col:lit(":") b:block() {
                make_except(kw, None, None, col, b)
            }

        rule except_star_block() -> ExceptStarHandler
            = kw:lit("except") star:lit("*") e:expression()
                a:(k:lit("as") n:name() {(k, n)})? col:lit(":") b:block() {
                    make_except_star(kw, star, e, a, col, b)
            }

        rule finally_block() -> Finally
            = kw:lit("finally") col:lit(":") b:block() {
                make_finally(kw, col, b)
            }


        // Match statement

        rule match_stmt() -> Match
            = kw:lit("match") subject:subject_expr() col:lit(":") tok(NL, "NEWLINE")
                i:tok(Indent, "INDENT") cases:case_block()+ d:tok(Dedent, "DEDENT") {
                    make_match(kw, subject, col, i, cases, d)
            }

        rule subject_expr() -> Expression
            = first:star_named_expression() c:comma() rest:star_named_expressions()? {
                Expression::Tuple(Box::new(
                    make_tuple_from_elements(first.with_comma(c), rest.unwrap_or_default()))
                )
            }
            / named_expression()

        rule case_block() -> MatchCase
            = kw:lit("case") pattern:patterns() guard:guard()? col:lit(":") body:block() {
                make_case(kw, pattern, guard, col, body)
            }

        rule guard() -> (TokenRef, Expression)
            = kw:lit("if") exp:named_expression() { (kw, exp) }

        rule patterns() -> MatchPattern
            = pats:open_sequence_pattern() {
                MatchPattern::Sequence(make_list_pattern(None, pats, None))
            }
            / pattern()

        rule pattern() -> MatchPattern
            = as_pattern()
            / or_pattern()

        rule as_pattern() -> MatchPattern
            = pat:or_pattern() kw:lit("as") target:pattern_capture_target() {
                make_as_pattern(Some(pat), Some(kw), Some(target))
            }

        rule or_pattern() -> MatchPattern
            = pats:separated(<closed_pattern()>, <lit("|")>) {
                make_or_pattern(pats.0, pats.1)
            }

        rule closed_pattern() -> MatchPattern
            = literal_pattern()
            / capture_pattern()
            / wildcard_pattern()
            / value_pattern()
            / group_pattern()
            / sequence_pattern()
            / mapping_pattern()
            / class_pattern()

        rule literal_pattern() -> MatchPattern
            = val:signed_number() !(lit("+") / lit("-")) { make_match_value(val) }
            / val:complex_number() { make_match_value(val) }
            / val:strings() { make_match_value(val.into()) }
            / n:lit("None") { make_match_singleton(make_name(n)) }
            / n:lit("True") { make_match_singleton(make_name(n)) }
            / n:lit("False") { make_match_singleton(make_name(n)) }

        rule literal_expr() -> Expression
            = val:signed_number() !(lit("+") / lit("-")) { val }
            / val:complex_number() { val }
            / val:strings() { val.into() }
            / n:lit("None") { Expression::Name(Box::new(make_name(n))) }
            / n:lit("True") { Expression::Name(Box::new(make_name(n))) }
            / n:lit("False") { Expression::Name(Box::new(make_name(n))) }

        rule complex_number() -> Expression
            = re:signed_real_number() op:(lit("+")/lit("-")) im:imaginary_number() {?
                make_binary_op(re, op, im).map_err(|_| "complex number")
            }

        rule signed_number() -> Expression
            = n:tok(Number, "number") { make_number(n) }
            / op:lit("-") n:tok(Number, "number") {?
                make_unary_op(op, make_number(n)).map_err(|_| "signed number")
            }

        rule signed_real_number() -> Expression
            = real_number()
            / op:lit("-") n:real_number() {?
                make_unary_op(op, n).map_err(|_| "signed real number")
            }

        rule real_number() -> Expression
            = n:tok(Number, "number") {? ensure_real_number(n) }

        rule imaginary_number() -> Expression
            = n:tok(Number, "number") {? ensure_imaginary_number(n) }

        rule capture_pattern() -> MatchPattern
            = t:pattern_capture_target() { make_as_pattern(None, None, Some(t)) }

        rule pattern_capture_target() -> Name
            = !lit("_") n:name() !(lit(".") / lit("(") / lit("=")) { n }

        rule wildcard_pattern() -> MatchPattern
            = lit("_") { make_as_pattern(None, None, None) }

        rule value_pattern() -> MatchPattern
            = v:attr() !(lit(".") / lit("(") / lit("=")) {
                make_match_value(v.into())
            }

        // In upstream attr and name_or_attr are mutually recursive, but rust-peg
        // doesn't support this yet.
        rule attr() -> NameOrAttribute
            = &(name() lit(".")) v:name_or_attr() { v }

        #[cache_left_rec]
        rule name_or_attr() -> NameOrAttribute
            = val:name_or_attr() d:lit(".") attr:name() {
                NameOrAttribute::A(Box::new(make_attribute(val.into(), d, attr)))
            }
            / n:name() { NameOrAttribute::N(Box::new(n)) }

        rule group_pattern() -> MatchPattern
            = l:lpar() pat:pattern() r:rpar() { pat.with_parens(l, r) }

        rule sequence_pattern() -> MatchPattern
            = l:lbrak() pats:maybe_sequence_pattern()? r:rbrak() {
                MatchPattern::Sequence(
                    make_list_pattern(Some(l), pats.unwrap_or_default(), Some(r))
                )
            }
            / l:lpar() pats:open_sequence_pattern()? r:rpar() {
                MatchPattern::Sequence(make_tuple_pattern(l, pats.unwrap_or_default(), r))
            }

        rule open_sequence_pattern() -> Vec<StarrableMatchSequenceElement>
            = pat:maybe_star_pattern() c:comma() pats:maybe_sequence_pattern()? {
                make_open_sequence_pattern(pat, c, pats.unwrap_or_default())
            }

        rule maybe_sequence_pattern() -> Vec<StarrableMatchSequenceElement>
            = pats:separated_trailer(<maybe_star_pattern()>, <comma()>) {
                comma_separate(pats.0, pats.1, pats.2)
            }

        rule maybe_star_pattern() -> StarrableMatchSequenceElement
            = s:star_pattern() { StarrableMatchSequenceElement::Starred(s) }
            / p:pattern() {
                StarrableMatchSequenceElement::Simple(
                    make_match_sequence_element(p)
                )
            }

        rule star_pattern() -> MatchStar
            = star:lit("*") t:pattern_capture_target() {make_match_star(star, Some(t))}
            / star:lit("*") t:wildcard_pattern() { make_match_star(star, None) }

        rule mapping_pattern() -> MatchPattern
            = l:lbrace() r:rbrace() {
                make_match_mapping(l, vec![], None, None, None, None, r)
            }
            / l:lbrace() rest:double_star_pattern() trail:comma()? r:rbrace() {
                make_match_mapping(l, vec![], None, Some(rest.0), Some(rest.1), trail, r)
            }
            / l:lbrace() items:items_pattern() c:comma() rest:double_star_pattern()
                trail:comma()? r:rbrace() {
                    make_match_mapping(l, items, Some(c), Some(rest.0), Some(rest.1), trail, r)
                }
            / l:lbrace() items:items_pattern() trail:comma()? r:rbrace() {
                make_match_mapping(l, items, trail, None, None, None, r)
            }

        rule items_pattern() -> Vec<MatchMappingElement>
            = pats:separated(<key_value_pattern()>, <comma()>) {
                comma_separate(pats.0, pats.1, None)
            }

        rule key_value_pattern() -> MatchMappingElement
            = key:(literal_expr() / a:attr() {a.into()}) colon:lit(":") pat:pattern() {
                make_match_mapping_element(key, colon, pat)
            }

        rule double_star_pattern() -> (TokenRef, Name)
            = star:lit("**") n:pattern_capture_target() { (star, n) }

        rule class_pattern() -> MatchPattern
            = cls:name_or_attr() l:lit("(") r:lit(")") {
                make_class_pattern(cls, l, vec![], None, vec![], None, r)
            }
            / cls:name_or_attr() l:lit("(") pats:positional_patterns() c:comma()? r:lit(")") {
                make_class_pattern(cls, l, pats, c, vec![], None, r)
            }
            / cls:name_or_attr() l:lit("(") kwds:keyword_patterns() c:comma()? r:lit(")") {
                make_class_pattern(cls, l, vec![], None, kwds, c, r)
            }
            / cls:name_or_attr() l:lit("(") pats:positional_patterns() c:comma()
                kwds:keyword_patterns() trail:comma()? r:lit(")") {
                    make_class_pattern(cls, l, pats, Some(c), kwds, trail, r)
            }

        rule positional_patterns() -> Vec<MatchSequenceElement>
            = pats:separated(<p:pattern() { make_match_sequence_element(p) }>, <comma()>) {
                comma_separate(pats.0, pats.1, None)
            }

        rule keyword_patterns() -> Vec<MatchKeywordElement>
            = pats:separated(<keyword_pattern()>, <comma()>) {
                comma_separate(pats.0, pats.1, None)
            }

        rule keyword_pattern() -> MatchKeywordElement
            = arg:name() eq:lit("=") value:pattern() {
                make_match_keyword_element(arg, eq, value)
            }

        // Expressions

        #[cache]
        rule expression() -> Expression
            = _conditional_expression()
            / lambdef()

        rule _conditional_expression() -> Expression
            = body:disjunction() i:lit("if") test:disjunction() e:lit("else") oe:expression() {
                Expression::IfExp(Box::new(make_ifexp(body, i, test, e, oe)))
            }
            / disjunction()

        rule yield_expr() -> Expression
            = y:lit("yield") f:lit("from") a:expression() {
                Expression::Yield(Box::new(make_yield(y, Some(f), Some(a))))
            }
            / y:lit("yield") a:star_expressions()? {
                Expression::Yield(Box::new(make_yield(y, None, a)))
            }

        rule star_expressions() -> Expression
            = first:star_expression()
                rest:(comma:comma() e:star_expression() { (comma, expr_to_element(e)) })+
                comma:comma()? {
                    Expression::Tuple(Box::new(make_tuple(expr_to_element(first), rest, comma, None, None)))
            }
            / e:star_expression() comma:comma() {
                Expression::Tuple(Box::new(make_tuple(expr_to_element(e), vec![], Some(comma), None, None)))
            }
            / star_expression()

        #[cache]
        rule star_expression() -> Expression
            = star:lit("*") e:bitwise_or() {
                Expression::StarredElement(Box::new(make_starred_element(star, expr_to_element(e))))
            }
            / expression()

        rule star_named_expressions() -> Vec<Element>
            = exps:separated_trailer(<star_named_expression()>, <comma()>) {
                comma_separate(exps.0, exps.1, exps.2)
            }

        rule star_named_expression() -> Element
            = star:lit("*") e:bitwise_or() {
                Element::Starred(Box::new(make_starred_element(star, expr_to_element(e))))
            }
            / e:named_expression() { expr_to_element(e) }

        rule named_expression() -> Expression
            = a:name() op:lit(":=") b:expression() {
                Expression::NamedExpr(Box::new(make_named_expr(a, op, b)))
            }
            / e:expression() !lit(":=") { e }

        #[cache]
        rule disjunction() -> Expression
            = a:conjunction() b:(or:lit("or") inner:conjunction() { (or, inner) })+ {?
                make_boolean_op(a, b).map_err(|e| "expected disjunction")
            }
            / conjunction()

        #[cache]
        rule conjunction() -> Expression
            = a:inversion() b:(and:lit("and") inner:inversion() { (and, inner) })+ {?
                make_boolean_op(a, b).map_err(|e| "expected conjunction")
            }
            / inversion()

        #[cache]
        rule inversion() -> Expression
            = not:lit("not") a:inversion() {?
                make_unary_op(not, a).map_err(|e| "expected inversion")
            }
            / comparison()

        // Comparison operators

        #[cache]
        rule comparison() -> Expression
            = a:bitwise_or() b:compare_op_bitwise_or_pair()+ { make_comparison(a, b) }
            / bitwise_or()

        // This implementation diverges slightly from CPython (3.9) to avoid bloating
        // the parser cache and increase readability.
        #[cache]
        rule compare_op_bitwise_or_pair() -> (CompOp, Expression)
            = _op_bitwise_or("==")
            / _op_bitwise_or("!=") // TODO: support barry_as_flufl
            / _op_bitwise_or("<=")
            / _op_bitwise_or("<")
            / _op_bitwise_or(">=")
            / _op_bitwise_or(">")
            / _op_bitwise_or2("not", "in")
            / _op_bitwise_or("in")
            / _op_bitwise_or2("is", "not")
            / _op_bitwise_or("is")

        rule _op_bitwise_or(o: &'static str) -> (CompOp, Expression)
            = op:lit(o) e:bitwise_or() {?
                make_comparison_operator(op)
                    .map(|op| (op, e))
                    .map_err(|_| "comparison")
            }

        rule _op_bitwise_or2(first: &'static str, second: &'static str) -> (CompOp, Expression)
            = f:lit(first) s:lit(second) e:bitwise_or() {?
                make_comparison_operator_2(f, s)
                    .map(|op| (op, e))
                    .map_err(|_| "comparison")
            }

        #[cache_left_rec]
        rule bitwise_or() -> Expression
            = a:bitwise_or() op:lit("|") b:bitwise_xor() {?
                make_binary_op(a, op, b).map_err(|e| "expected bitwise_or")
            }
            / bitwise_xor()

        #[cache_left_rec]
        rule bitwise_xor() -> Expression
            = a:bitwise_xor() op:lit("^") b:bitwise_and() {?
                make_binary_op(a, op, b).map_err(|e| "expected bitwise_xor")
            }
            / bitwise_and()

        #[cache_left_rec]
        rule bitwise_and() -> Expression
            = a:bitwise_and() op:lit("&") b:shift_expr() {?
                make_binary_op(a, op, b).map_err(|e| "expected bitwise_and")
            }
            / shift_expr()

        #[cache_left_rec]
        rule shift_expr() -> Expression
            = a:shift_expr() op:lit("<<") b:sum() {?
                make_binary_op(a, op, b).map_err(|e| "expected shift_expr")
            }
            / a:shift_expr() op:lit(">>") b:sum() {?
                make_binary_op(a, op, b).map_err(|e| "expected shift_expr")
            }
            / sum()

        #[cache_left_rec]
        rule sum() -> Expression
            = a:sum() op:lit("+") b:term() {?
                make_binary_op(a, op, b).map_err(|e| "expected sum")
            }
            / a:sum() op:lit("-") b:term() {?
                make_binary_op(a, op, b).map_err(|e| "expected sum")
            }
            / term()

        #[cache_left_rec]
        rule term() -> Expression
            = a:term() op:lit("*") b:factor() {?
                make_binary_op(a, op, b).map_err(|e| "expected term")
            }
            / a:term() op:lit("/") b:factor() {?
                make_binary_op(a, op, b).map_err(|e| "expected term")
            }
            / a:term() op:lit("//") b:factor() {?
                make_binary_op(a, op, b).map_err(|e| "expected term")
            }
            / a:term() op:lit("%") b:factor() {?
                make_binary_op(a, op, b).map_err(|e| "expected term")
            }
            / a:term() op:lit("@") b:factor() {?
                make_binary_op(a, op, b).map_err(|e| "expected term")
            }
            / factor()

        #[cache]
        rule factor() -> Expression
            = op:lit("+") a:factor() {?
                make_unary_op(op, a).map_err(|e| "expected factor")
            }
            / op:lit("-") a:factor() {?
                make_unary_op(op, a).map_err(|e| "expected factor")
            }
            / op:lit("~") a:factor() {?
                make_unary_op(op, a).map_err(|e| "expected factor")
            }
            / power()

        rule power() -> Expression
            = a:await_primary() op:lit("**") b:factor() {?
                make_binary_op(a, op, b).map_err(|e| "expected power")
            }
            / await_primary()

        // Primary elements

        rule await_primary() -> Expression
            = aw:tok(AWAIT, "AWAIT") e:primary() {
                Expression::Await(Box::new(make_await(aw, e)))
            }
            / primary()

        #[cache_left_rec]
        rule primary() -> Expression
            = v:primary() dot:lit(".") attr:name() {
                Expression::Attribute(Box::new(make_attribute(v, dot, attr)))
            }
            / a:primary() b:genexp() {
                Expression::Call(Box::new(make_genexp_call(a, b)))
            }
            / f:primary() lpar:lit("(") arg:arguments()? rpar:lit(")") {
                Expression::Call(Box::new(make_call(f, lpar, arg.unwrap_or_default(), rpar)))
            }
            / v:primary() lbrak:lbrak() s:slices() rbrak:rbrak() {
                Expression::Subscript(Box::new(make_subscript(v, lbrak, s, rbrak)))
            }
            / atom()

        rule slices() -> Vec<SubscriptElement>
            = s:slice() !lit(",") { vec![SubscriptElement { slice: s, comma: None }] }
            / slices:separated_trailer(<slice()>, <comma()>) {
                make_slices(slices.0, slices.1, slices.2)
            }

        rule slice() -> BaseSlice
            = l:expression()? col:lit(":") u:expression()?
                rest:(c:lit(":") s:expression()? {(c, s)})? {
                    make_slice(l, col, u, rest)
            }
            / v:expression() { make_index(v) }

        rule atom() -> Expression
            = n:name() { Expression::Name(Box::new(n)) }
            / n:lit("True") { Expression::Name(Box::new(make_name(n))) }
            / n:lit("False") { Expression::Name(Box::new(make_name(n))) }
            / n:lit("None") { Expression::Name(Box::new(make_name(n))) }
            / &(tok(STRING, "") / tok(FStringStart, "")) s:strings() {s.into()}
            / n:tok(Number, "NUMBER") { make_number(n) }
            / &lit("(") e:(tuple() / group() / (g:genexp() {Expression::GeneratorExp(Box::new(g))})) {e}
            / &lit("[") e:(list() / listcomp()) {e}
            / &lit("{") e:(dict() / set() / dictcomp() / setcomp()) {e}
            / lit("...") { Expression::Ellipsis(Box::new(Ellipsis {lpar: vec![], rpar: vec![]}))}

        rule group() -> Expression
            = lpar:lpar() e:(yield_expr() / named_expression()) rpar:rpar() {
                e.with_parens(lpar, rpar)
            }

        // Lambda functions

        rule lambdef() -> Expression
            = kw:lit("lambda") p:lambda_params()? c:lit(":") b:expression() {
                Expression::Lambda(Box::new(make_lambda(kw, p.unwrap_or_default(), c, b)))
            }

        rule lambda_params() -> Parameters
            = lambda_parameters()

        // lambda_parameters etc. duplicates parameters but without annotations or type
        // comments, and if there's no comma after a parameter, we expect a colon, not a
        // close parenthesis.

        rule lambda_parameters() -> Parameters
            = a:lambda_slash_no_default() b:lambda_param_no_default()*
                c:lambda_param_with_default()* d:lambda_star_etc()? {
                    make_parameters(Some(a), concat(b, c), d)
            }
            / a:lambda_slash_with_default() b:lambda_param_with_default()*
                d:lambda_star_etc()? {
                    make_parameters(Some(a), b, d)
            }
            / a:lambda_param_no_default()+ b:lambda_param_with_default()*
                d:lambda_star_etc()? {
                    make_parameters(None, concat(a, b), d)
            }
            / a:lambda_param_with_default()+ d:lambda_star_etc()? {
                make_parameters(None, a, d)
            }
            / d:lambda_star_etc() {
                make_parameters(None, vec![], Some(d))
            }

        rule lambda_slash_no_default() -> (Vec<Param>, ParamSlash)
            = a:lambda_param_no_default()+ slash:lit("/") com:comma() {
                (a, ParamSlash { comma: Some(com) } )
            }
            / a:lambda_param_no_default()+ slash:lit("/") &lit(":") {
                (a, ParamSlash { comma: None })
            }

        rule lambda_slash_with_default() -> (Vec<Param>, ParamSlash)
            = a:lambda_param_no_default()* b:lambda_param_with_default()+ slash:lit("/") c:comma(){
                (concat(a, b), ParamSlash { comma: Some(c) })
            }
            / a:lambda_param_no_default()* b:lambda_param_with_default()+ slash:lit("/") &lit(":") {
                (concat(a, b), ParamSlash { comma: None })
            }

        rule lambda_star_etc() -> StarEtc
            = star:lit("*") a:lambda_param_no_default()
                b:lambda_param_maybe_default()* kw:lambda_kwds()? {
                    StarEtc(Some(StarArg::Param(
                        Box::new(add_param_star(a, star))
                    )), b, kw)
            }
            / lit("*") c:comma() b:lambda_param_maybe_default()+ kw:lambda_kwds()? {
                StarEtc(Some(StarArg::Star(Box::new(ParamStar {comma: c}))), b, kw)
            }
            / kw:lambda_kwds() { StarEtc(None, vec![], Some(kw)) }

        rule lambda_kwds() -> Param
            = star:lit("**") a:lambda_param_no_default() {
                add_param_star(a, star)
            }

        rule lambda_param_no_default() -> Param
            = a:lambda_param() c:lit(",") {
                add_param_default(a, None, Some(c))
            }
            / a:lambda_param() &lit(":") {a}

        rule lambda_param_with_default() -> Param
            = a:lambda_param() def:default() c:lit(",") {
                add_param_default(a, Some(def), Some(c))
            }
            / a:lambda_param() def:default() &lit(":") {
                add_param_default(a, Some(def), None)
            }

        rule lambda_param_maybe_default() -> Param
            = a:lambda_param() def:default()? c:lit(",") {
                add_param_default(a, def, Some(c))
            }
            / a:lambda_param() def:default()? &lit(":") {
                add_param_default(a, def, None)
            }

        rule lambda_param() -> Param
            = name:name() { Param { name, ..Default::default() } }

        // Literals

        // todo deal with + infinite loop here
        rule strings() -> String
            = s:(str:tok(STRING, "STRING") t:&_ {( make_string(str), t) }
                / str:fstring() t:&_ {(String::Formatted(str), t)}) {
                make_strings(s)
            }

        rule list() -> Expression
            = lbrak:lbrak() e:star_named_expressions()? rbrak:rbrak() {
                Expression::List(Box::new(
                    make_list(lbrak, e.unwrap_or_default(), rbrak))
                )
            }

        rule tuple() -> Expression
            = lpar:lpar() first:star_named_expression() &lit(",")
                rest:(c:comma() e:star_named_expression() {(c, e)})*
                trailing_comma:comma()? rpar:rpar() {
                    Expression::Tuple(Box::new(
                        make_tuple(first, rest, trailing_comma, Some(lpar), Some(rpar))
                    ))
            }
            / lpar:lpar() rpar:lit(")") {
                Expression::Tuple(Box::new(Tuple::default().with_parens(
                    lpar, RightParen { whitespace_before: Default::default(), rpar_tok: rpar }
                )))}

        rule set() -> Expression
            = lbrace:lbrace() e:star_named_expressions()? rbrace:rbrace() {
                Expression::Set(Box::new(make_set(lbrace, e.unwrap_or_default(), rbrace)))
            }

        // Dicts

        rule dict() -> Expression
            = lbrace:lbrace() els:double_starred_keypairs()? rbrace:rbrace() {
                Expression::Dict(Box::new(make_dict(lbrace, els.unwrap_or_default(), rbrace)))
            }


        rule double_starred_keypairs() -> Vec<DictElement>
            = pairs:separated_trailer(<double_starred_kvpair()>, <comma()>) {
                    make_double_starred_keypairs(pairs.0, pairs.1, pairs.2)
            }

        rule double_starred_kvpair() -> DictElement
            = s:lit("**") e:bitwise_or() {
                DictElement::Starred(make_double_starred_element(s, e))
            }
            / k:kvpair() { make_dict_element(k) }

        rule kvpair() -> (Expression, TokenRef, Expression)
            = k:expression() colon:lit(":") v:expression() { (k, colon, v) }

        // Comprehensions & generators

        rule for_if_clauses() -> CompFor
            = c:for_if_clause()+ { merge_comp_fors(c) }

        rule for_if_clause() -> CompFor
            = asy:_async() f:lit("for") tgt:star_targets() i:lit("in")
                iter:disjunction() ifs:_comp_if()* {
                    make_for_if(Some(asy), f, tgt, i, iter, ifs)
            }
            / f:lit("for") tgt:star_targets() i:lit("in")
            iter:disjunction() ifs:_comp_if()* {
                make_for_if(None, f, tgt, i, iter, ifs)
            }

        rule _comp_if() -> CompIf
            = kw:lit("if") cond:disjunction() {
                make_comp_if(kw, cond)
            }

        rule listcomp() -> Expression
            = lbrak:lbrak() elt:named_expression() comp:for_if_clauses() rbrak:rbrak() {
                Expression::ListComp(Box::new(make_list_comp(lbrak, elt, comp, rbrak)))
            }

        rule setcomp() -> Expression
            = l:lbrace() elt:named_expression() comp:for_if_clauses() r:rbrace() {
                Expression::SetComp(Box::new(make_set_comp(l, elt, comp, r)))
            }

        rule genexp() -> GeneratorExp
            = lpar:lpar() g:_bare_genexp() rpar:rpar() {
                g.with_parens(lpar, rpar)
            }

        rule _bare_genexp() -> GeneratorExp
            = elt:named_expression() comp:for_if_clauses() {
                make_bare_genexp(elt, comp)
            }

        rule dictcomp() -> Expression
            = lbrace:lbrace() elt:kvpair() comp:for_if_clauses() rbrace:rbrace() {
                Expression::DictComp(Box::new(make_dict_comp(lbrace, elt, comp, rbrace)))
            }

        // Function call arguments

        rule arguments() -> Vec<Arg>
            = a:args() trail:comma()? &lit(")") {add_arguments_trailing_comma(a, trail)}

        rule args() -> Vec<Arg>
            = first:_posarg()
                rest:(c:comma() a:_posarg() {(c, a)})*
                kw:(c:comma() k:kwargs() {(c, k)})? {
                    let (trail, kw) = kw.map(|(x,y)| (Some(x), Some(y))).unwrap_or((None, None));
                    concat(
                        comma_separate(first, rest, trail),
                        kw.unwrap_or_default(),
                    )
            }
            / kwargs()

        rule _posarg() -> Arg
            = a:(starred_expression() / e:named_expression() { make_arg(e) })
                !lit("=") { a }

        rule kwargs() -> Vec<Arg>
            = sitems:separated(<kwarg_or_starred()>, <comma()>)
                scomma:comma()
                ditems:separated(<kwarg_or_double_starred()>, <comma()>) {
                    concat(
                        comma_separate(sitems.0, sitems.1, Some(scomma)),
                        comma_separate(ditems.0, ditems.1, None),
                    )
            }
            / items:separated(<kwarg_or_starred()>, <comma()>) {
                    comma_separate(items.0, items.1, None)
            }
            / items:separated(<kwarg_or_double_starred()>, <comma()>) {
                    comma_separate(items.0, items.1, None)
            }

        rule starred_expression() -> Arg
            = star:lit("*") e:expression() { make_star_arg(star, e) }

        rule kwarg_or_starred() -> Arg
            = _kwarg()
            / starred_expression()

        rule kwarg_or_double_starred() -> Arg
            = _kwarg()
            / star:lit("**") e:expression() { make_star_arg(star, e) }

        rule _kwarg() -> Arg
            = n:name() eq:lit("=") v:expression() {
                make_kwarg(n, eq, v)
            }

        // Assignment targets
        // Generic targets

        rule star_targets() -> AssignTargetExpression
            = a:star_target() !lit(",") {a}
            / targets:separated_trailer(<t:star_target() {assign_target_to_element(t)}>, <comma()>) {
                AssignTargetExpression::Tuple(Box::new(
                    make_tuple(targets.0, targets.1, targets.2, None, None)
                ))
            }

        rule star_targets_list_seq() -> Vec<Element>
            = targets:separated_trailer(<t:star_target() { assign_target_to_element(t) }>, <comma()>) {
                comma_separate(targets.0, targets.1, targets.2)
            }

        // This differs from star_targets below because it requires at least two items
        // in the tuple
        rule star_targets_tuple_seq() -> Tuple
            = first:(t:star_target() {assign_target_to_element(t)})
                rest:(c:comma() t:star_target() {(c, assign_target_to_element(t))})+
                trail:comma()? {
                    make_tuple(first, rest, trail, None, None)
            }
            / t:star_target() trail:comma()? {
                make_tuple(assign_target_to_element(t), vec![], trail, None, None)
            }

        #[cache]
        rule star_target() -> AssignTargetExpression
            = star:lit("*") !lit("*") t:star_target() {
                AssignTargetExpression::StarredElement(Box::new(
                    make_starred_element(star, assign_target_to_element(t))
                ))
            }
            / target_with_star_atom()

        #[cache]
        rule target_with_star_atom() -> AssignTargetExpression
            = a:t_primary() dot:lit(".") n:name() !t_lookahead() {
                AssignTargetExpression::Attribute(Box::new(make_attribute(a, dot, n)))
            }
            / a:t_primary() lbrak:lbrak() s:slices() rbrak:rbrak() !t_lookahead() {
                AssignTargetExpression::Subscript(Box::new(
                    make_subscript(a, lbrak, s, rbrak)
                ))
            }
            / a:star_atom() {a}

        rule star_atom() -> AssignTargetExpression
            = a:name() { AssignTargetExpression::Name(Box::new(a)) }
            / lpar:lpar() a:target_with_star_atom() rpar:rpar() { a.with_parens(lpar, rpar) }
            / lpar:lpar() a:star_targets_tuple_seq()? rpar:rpar() {
               AssignTargetExpression::Tuple(Box::new(
                   a.unwrap_or_default().with_parens(lpar, rpar)
               ))
            }
            / lbrak:lbrak() a:star_targets_list_seq()? rbrak:rbrak() {
                AssignTargetExpression::List(Box::new(
                    make_list(lbrak, a.unwrap_or_default(), rbrak)
                ))
            }

        rule single_target() -> AssignTargetExpression
            = single_subscript_attribute_target()
            / n:name() { AssignTargetExpression::Name(Box::new(n)) }
            / lpar:lpar() t:single_target() rpar:rpar() { t.with_parens(lpar, rpar) }

        rule single_subscript_attribute_target() -> AssignTargetExpression
            = a:t_primary() dot:lit(".") n:name() !t_lookahead() {
                AssignTargetExpression::Attribute(Box::new(make_attribute(a, dot, n)))
            }
            / a:t_primary() lbrak:lbrak() s:slices() rbrak:rbrak() !t_lookahead() {
                AssignTargetExpression::Subscript(Box::new(
                    make_subscript(a, lbrak, s, rbrak)
                ))
            }


        #[cache_left_rec]
        rule t_primary() -> Expression
            = value:t_primary() dot:lit(".") attr:name() &t_lookahead() {
                Expression::Attribute(Box::new(make_attribute(value, dot, attr)))
            }
            / v:t_primary() l:lbrak() s:slices() r:rbrak() &t_lookahead() {
                Expression::Subscript(Box::new(make_subscript(v, l, s, r)))
            }
            / f:t_primary() gen:genexp() &t_lookahead() {
                Expression::Call(Box::new(make_genexp_call(f, gen)))
            }
            / f:t_primary() lpar:lit("(") arg:arguments()? rpar:lit(")") &t_lookahead() {
                Expression::Call(Box::new(make_call(f, lpar, arg.unwrap_or_default(), rpar)))
            }
            / a:atom() &t_lookahead() {a}

        rule t_lookahead() -> ()
            = (lit("(") / lit("[") / lit(".")) {}

        // Targets for del statements

        rule del_targets() -> Vec<Element>
            = t:separated_trailer(<u:del_target() {u.into()}>, <comma()>) {
                comma_separate(t.0, t.1, t.2)
            }

        rule del_target() -> DelTargetExpression
            = a:t_primary() d:lit(".") n:name() !t_lookahead() {
                DelTargetExpression::Attribute(Box::new(make_attribute(a, d, n)))
            }
            / a:t_primary() lbrak:lbrak() s:slices() rbrak:rbrak() !t_lookahead() {
                DelTargetExpression::Subscript(Box::new(
                    make_subscript(a, lbrak, s, rbrak)
                ))
            }
            / del_t_atom()

        rule del_t_atom() -> DelTargetExpression
            = n:name() { DelTargetExpression::Name(Box::new(n)) }
            / l:lpar() d:del_target() r:rpar() { d.with_parens(l, r) }
            / l:lpar() d:del_targets()? r:rpar() {
                make_del_tuple(Some(l), d.unwrap_or_default(), Some(r))
            }
            / l:lbrak() d:del_targets()? r:rbrak() {
                DelTargetExpression::List(Box::new(
                    make_list(l, d.unwrap_or_default(), r)
                ))
            }

        // F-strings

        rule fstring() -> FormattedString
            = start:tok(FStringStart, "f\"")
                parts:(_f_string() / _f_replacement())*
                end:tok(FStringEnd, "\"") {
                    make_fstring(start.string, parts, end.string)
            }

        rule _f_string() -> FormattedStringContent
            = t:tok(FStringString, "f-string contents") {
                FormattedStringContent::Text(FormattedStringText { value: t.text })
            }

        rule _f_replacement() -> FormattedStringContent
            = lb:lit("{") e:_f_expr() eq:lit("=")?
                conv:(t:lit("!") c:_f_conversion() {(t,c)})?
                spec:(t:lit(":") s:_f_spec() {(t,s)})?
                rb:lit("}") {
                    FormattedStringContent::Expression(Box::new(
                        make_fstring_expression(lb, e, eq, conv, spec, rb)
                    ))
            }

        rule _f_expr() -> Expression
            = (g:_bare_genexp() {Expression::GeneratorExp(Box::new(g))})
            / star_expressions()
            / yield_expr()

        rule _f_conversion() -> &'a str
            = lit("r") {"r"} / lit("s") {"s"} / lit("a") {"a"}

        rule _f_spec() -> Vec<FormattedStringContent>
            = (_f_string() / _f_replacement())*

        // CST helpers

        rule comma() -> Comma
            = c:lit(",") { make_comma(c) }

        rule dots() -> Vec<Dot>
            = ds:((dot:lit(".") { make_dot(dot) })+
                / tok:lit("...") {
                    vec![make_dot(tok.clone()), make_dot(tok.clone()), make_dot(tok.clone())]}
            )+ { ds.into_iter().flatten().collect() }

        rule lpar() -> LeftParen
            = a:lit("(") { make_lpar(a) }

        rule rpar() -> RightParen
            = a:lit(")") { make_rpar(a) }

        rule lbrak() -> LeftSquareBracket
            = tok:lit("[") { make_left_bracket(tok) }

        rule rbrak() -> RightSquareBracket
            = tok:lit("]") { make_right_bracket(tok) }

        rule lbrace() -> LeftCurlyBrace
            = tok:lit("{") { make_left_brace(tok) }

        rule rbrace() -> RightCurlyBrace
            = tok:lit("}") { make_right_brace(tok) }

        /// matches any token, not just whitespace
        rule _() -> TokenRef
            = [t] { t }



        //Utility rules
        rule lit(lit:  &'static str) -> TokenRef
        = [t] {? if t.text == lit {Ok(t)} else {Err(lit)}}

        rule tok(tok: TType, err: &'static str) -> TokenRef
        = [t] {? if t.r#type == tok { Ok(t)} else {Err(err)} }

        rule name() -> Name
            = !( lit("False") / lit("None") / lit("True") / lit("and") / lit("as") / lit("assert") / lit("async") / lit("await")
                / lit("break") / lit("class") / lit("continue") / lit("def") / lit("del") / lit("elif") / lit("else")
                / lit("except") / lit("finally") / lit("for") / lit("from") / lit("global") / lit("if") / lit("import")
                / lit("in") / lit("is") / lit("lambda") / lit("nonlocal") / lit("not") / lit("or") / lit("pass") / lit("raise")
                / lit("return") / lit("try") / lit("while") / lit("with") / lit("yield")
            )
            t:tok(NameTok, "NAME") {make_name(t)}

        rule _async() -> TokenRef
            = tok(Async, "ASYNC")

        rule separated_trailer<El, Sep>(el: rule<El>, sep: rule<Sep>) -> (El, Vec<(Sep, El)>, Option<Sep>)
            = e:el() rest:(s:sep() e:el() {(s, e)})* trailer:sep()? {(e, rest, trailer)}

        rule separated<El, Sep>(el: rule<El>, sep: rule<Sep>) -> (El, Vec<(Sep, El)>)
            = e:el() rest:(s:sep() e:el() {(s, e)})* {(e, rest)}



        rule traced<T>(e: rule<T>) -> T =
            &(_* {
                #[cfg(feature = "trace")]
                {
                    println!("[PEG_INPUT_START]");
                    println!("{}", input);
                    println!("[PEG_TRACE_START]");
                }
            })
            e:e()? {?
                #[cfg(feature = "trace")]
                println!("[PEG_TRACE_STOP]");
                e.ok_or("")
            }



    } //end python grammar
} //end parse!

fn make_addition(first: TokenRef, second: TokenRef) -> ValueNode{
    println!("{:?},{:?}", first, second);
    let a: u32 = first.text.parse().expect("number");
    let b: u32 = second.text.parse().expect("second number");
    return ValueNode{ result: a.saturating_add(b) };
}

#[cfg(test)]
mod tests {
    use crate::parser::grammar::{python, TokenRef, TokVec};
    use crate::tokenizer::Token;
    use crate::tokenizer::TType::{Op, Number};
    use std::rc::Rc;


    #[test]
    fn basic() {
        let data = TokVec(
            vec![
                Rc::new(Token::quick(Number, 1, 1, 2, "1")),
                Rc::new(Token::quick(Op, 1, 3, 4, "+")),
                Rc::new(Token::quick(Number, 1, 5, 6, "1")), ],
        );


        let result = python::statement(&data);
        match result {
            Ok(value) => println!("success {}", value.result),
            Err(issue) => println!("problem {}", issue),
        }

        assert_eq!(1, 2);


    }

}