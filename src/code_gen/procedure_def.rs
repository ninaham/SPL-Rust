use crate::absyn::{
    absyn::{Expression, Statement, Variable},
    array_access::ArrayAccess,
    assign_statement::AssignStatement,
    binary_expression::Operator,
    call_statement::CallStatement,
    if_statement::IfStatement,
    procedure_definition::ProcedureDefinition,
    unary_expression::UnaryOperator,
    while_statement::WhileStatement,
};

use super::{Quadrupel, QuadrupelArg, QuadrupelOp, QuadrupelResult, QuadrupelVar, Tac};

impl<'a> Tac {
    pub(super) fn eval_proc_def(&mut self, proc_def: &'a ProcedureDefinition) {
        let quadrupel = Quadrupel::new();
        let mut new_quad: Quadrupel = quadrupel;
        new_quad.result = QuadrupelResult::Label(proc_def.name.clone());
        self.quadrupels.push(new_quad);
        for statement in &proc_def.body {
            self.eval_statement(statement);
        }
    }

    fn eval_statement(&mut self, statement: &'a Statement) {
        match statement {
            Statement::AssignStatement(assign) => {
                self.eval_assign_statement(assign.as_ref());
            }
            Statement::IfStatement(if_state) => {
                self.eval_if_statement(if_state.as_ref());
            }
            Statement::WhileStatement(while_state) => {
                self.eval_while_statement(while_state.as_ref());
            }
            Statement::CallStatement(call_state) => {
                self.eval_call_statement(call_state.as_ref());
            }
            Statement::CompoundStatement(inner) => {
                for statement in inner {
                    self.eval_statement(statement.as_ref());
                }
            }
            Statement::EmptyStatement => {}
        }
    }

    fn eval_assign_statement(&mut self, assign: &'a AssignStatement) {
        #![expect(clippy::similar_names)]
        let mut assign_quad;
        let val = self.eval_expression(&assign.value);

        match &assign.target {
            Variable::NamedVariable(name) => {
                match val {
                    Expr::Quad(quad) => {
                        assign_quad = quad;
                    }
                    Expr::Arg(arg) => {
                        assign_quad = Quadrupel::new();
                        assign_quad.op = QuadrupelOp::Assign;
                        assign_quad.arg1 = arg;
                    }
                }
                assign_quad.result = QuadrupelResult::Var(QuadrupelVar::Spl(name.to_string()));
            }
            Variable::ArrayAccess(array_access) => {
                let (var, offset) = self.eval_array_access(array_access);

                assign_quad = Quadrupel::new();
                assign_quad.op = QuadrupelOp::ArrayStore;
                assign_quad.result = QuadrupelResult::Var(var);
                assign_quad.arg2 = self.into_tmp(offset);
                assign_quad.arg1 = self.into_tmp(val);
            }
        }
        self.quadrupels.push(assign_quad);
    }

    fn eval_array_access(&mut self, array_access: &ArrayAccess) -> (QuadrupelVar, Expr) {
        let index = self.eval_expression(&array_access.index);
        let base_size = array_access.typ.as_ref().unwrap().base_type.get_byte_size();
        let var;
        let mut offset = self.emit_expression_bin(
            Operator::Mul,
            index,
            Expr::Arg(QuadrupelArg::Const(base_size)),
        );

        match &array_access.array {
            Variable::NamedVariable(name) => {
                var = QuadrupelVar::Spl(name.to_string());
            }
            Variable::ArrayAccess(inner) => {
                let (inner_var, inner_offset) = self.eval_array_access(inner);
                var = inner_var;
                offset = self.emit_expression_bin(Operator::Add, offset, inner_offset);
            }
        }

        (var, offset)
    }

    fn eval_if_statement(&mut self, if_state: &'a IfStatement) {
        let else_label = self.create_label(None);
        let end_label = self.create_label(None);
        let mut if_quad = Quadrupel::new();
        let ex = &if_state.condition;
        match ex {
            Expression::BinaryExpression(binex) => {
                let left = self.eval_expression(&binex.left);
                let right = self.eval_expression(&binex.right);

                if_quad.op = QuadrupelOp::from(binex.operator).inv();
                if_quad.arg1 = self.into_tmp(left);
                if_quad.arg2 = self.into_tmp(right);
                if_quad.result = else_label.clone();
                self.quadrupels.push(if_quad);
            }
            _ => panic!("mistake in 'if' expression!"),
        }
        // begin then
        self.eval_statement(&if_state.then_branch);
        if if_state.else_branch.is_some() {
            let mut quad = Quadrupel::new();
            quad.op = QuadrupelOp::Goto;
            quad.result = end_label.clone();
            self.quadrupels.push(quad);
        }
        // end then

        // begin else
        self.emit_label(else_label);
        if let Some(state) = &if_state.else_branch {
            self.eval_statement(state);
        }
        // end else
        self.emit_label(end_label);
    }

    fn eval_while_statement(&mut self, while_state: &'a WhileStatement) {
        let jmp_label = self.create_label(None);
        let while_label = self.create_label(None);
        self.emit_label(while_label.clone());
        let mut while_quad = Quadrupel::new();
        let ex = &while_state.condition;
        match ex {
            Expression::BinaryExpression(binex) => {
                let left = self.eval_expression(&binex.left);
                let right = self.eval_expression(&binex.right);

                while_quad.op = QuadrupelOp::from(binex.operator).inv();
                while_quad.arg1 = self.into_tmp(left);
                while_quad.arg2 = self.into_tmp(right);
                while_quad.result = jmp_label.clone();
                self.quadrupels.push(while_quad);
            }
            _ => panic!("mistake in 'while' expression!"),
        }
        self.eval_statement(&while_state.body);
        self.quadrupels.push(Quadrupel {
            op: QuadrupelOp::Goto,
            arg1: QuadrupelArg::Empty,
            arg2: QuadrupelArg::Empty,
            result: while_label,
        });
        self.emit_label(jmp_label);
    }

    fn eval_call_statement(&mut self, call_state: &'a CallStatement) {
        let mut count = 0;
        let name = call_state.name.clone();
        for param in &call_state.arguments {
            count += 1;
            let param = self.eval_expression(param);
            let mut quad = Quadrupel::new();
            quad.op = QuadrupelOp::Param;
            quad.arg1 = self.into_tmp(param);
            self.quadrupels.push(quad);
        }
        let mut quad = Quadrupel::new();
        quad.op = QuadrupelOp::Call;
        quad.arg1 = QuadrupelArg::Var(QuadrupelVar::Spl(name));
        quad.arg2 = QuadrupelArg::Const(count);
        self.quadrupels.push(quad);
    }

    fn eval_expression(&mut self, exp: &Expression) -> Expr {
        match exp {
            Expression::BinaryExpression(exp) => {
                let left = self.eval_expression(&exp.left);
                let right = self.eval_expression(&exp.right);
                self.emit_expression_bin(exp.operator, left, right)
            }
            Expression::UnaryExpression(exp) => {
                let left = self.eval_expression(&exp.operand);
                self.emit_expression_un(exp.operator, left)
            }
            Expression::IntLiteral(val) => Expr::Arg(QuadrupelArg::Const(*val)),
            Expression::VariableExpression(var) => self.eval_expression_var(var),
        }
    }

    fn eval_expression_var(&mut self, var: &Variable) -> Expr {
        match var {
            Variable::NamedVariable(name) => {
                Expr::Arg(QuadrupelArg::Var(QuadrupelVar::Spl(name.to_string())))
            }
            Variable::ArrayAccess(array_access) => {
                let (var, offset) = self.eval_array_access(array_access);
                self.emit_expression_arr_acc(var, offset)
            }
        }
    }

    fn emit_expression_arr_acc(&mut self, array_var: QuadrupelVar, offset: Expr) -> Expr {
        let mut quad = Quadrupel::new();
        quad.op = QuadrupelOp::ArrayLoad;
        quad.arg1 = QuadrupelArg::Var(array_var);
        quad.arg2 = self.into_tmp(offset);
        Expr::Quad(quad)
    }

    fn emit_expression_bin(&mut self, op: Operator, left: Expr, right: Expr) -> Expr {
        let mut quad = Quadrupel::new();
        quad.op = op.into();
        quad.arg1 = self.into_tmp(left);
        quad.arg2 = self.into_tmp(right);
        Expr::Quad(quad)
    }

    fn emit_expression_un(&mut self, op: UnaryOperator, left: Expr) -> Expr {
        let mut quad = Quadrupel::new();
        quad.op = op.into();
        quad.arg1 = self.into_tmp(left);
        Expr::Quad(quad)
    }

    fn create_label(&mut self, name: Option<String>) -> QuadrupelResult {
        let label: String;
        if let Some(name) = name {
            label = name;
        } else {
            label = format!("L{}", self.label_num);
            self.label_num += 1;
        }
        QuadrupelResult::Label(label)
    }

    fn emit_label(&mut self, label: QuadrupelResult) {
        assert!(matches!(label, QuadrupelResult::Label(_)));
        let mut new_quad: Quadrupel = Quadrupel::new();
        new_quad.result = label;
        self.quadrupels.push(new_quad);
    }

    const fn create_tmp_var(&mut self) -> QuadrupelVar {
        let n = self.temp_var_count;
        self.temp_var_count += 1;

        QuadrupelVar::Tmp(n)
    }

    #[expect(clippy::wrong_self_convention)]
    fn into_tmp(&mut self, expr: Expr) -> QuadrupelArg {
        match expr {
            Expr::Quad(mut quad) => {
                let tmp = self.create_tmp_var();
                quad.result = QuadrupelResult::Var(tmp.clone());
                self.quadrupels.push(quad);
                QuadrupelArg::Var(tmp)
            }
            Expr::Arg(arg) => arg,
        }
    }
}

impl Quadrupel {
    const fn new() -> Self {
        Self {
            op: super::QuadrupelOp::Default,
            arg1: QuadrupelArg::Empty,
            arg2: QuadrupelArg::Empty,
            result: QuadrupelResult::Empty,
        }
    }
}

enum Expr {
    Quad(Quadrupel),
    Arg(QuadrupelArg),
}
