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

impl<'a> Tac<'a> {
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
        let mut assign_quad = Quadrupel::new();
        match &assign.target {
            Variable::NamedVariable(name) => {
                assign_quad.op = QuadrupelOp::Assign;
                assign_quad.result = QuadrupelResult::Var(QuadrupelVar::Spl(name.to_string()));
            }
            Variable::ArrayAccess(array_access) => {
                assign_quad.op = QuadrupelOp::ArrayStore;
                let (var, offset) = self.eval_array_access(array_access);
                assign_quad.result = QuadrupelResult::Var(var);
                assign_quad.arg2 = QuadrupelArg::Var(offset);
            }
        }

        assign_quad.arg1 = self.eval_expression(&assign.value);
        self.quadrupels.push(assign_quad);
    }

    fn eval_array_access(&mut self, array_access: &ArrayAccess) -> (QuadrupelVar, QuadrupelVar) {
        let index = self.eval_expression(&array_access.index);
        let base_size = array_access.typ.as_ref().unwrap().base_type.get_byte_size();
        let var;
        let mut offset =
            self.emit_expression_bin(Operator::Mul, index, QuadrupelArg::Const(base_size));

        match &array_access.array {
            Variable::NamedVariable(name) => {
                var = QuadrupelVar::Spl(name.to_string());
            }
            Variable::ArrayAccess(inner) => {
                let (inner_var, inner_offset) = self.eval_array_access(inner);
                var = inner_var;
                offset = self.emit_expression_bin(
                    Operator::Add,
                    QuadrupelArg::Var(offset),
                    QuadrupelArg::Var(inner_offset),
                );
            }
        }

        (var, offset)
    }

    fn eval_if_statement(&mut self, if_state: &'a IfStatement) {
        let jmp_label = self.label_num;
        self.label_num += 1;
        let mut if_quad = Quadrupel::new();
        let ex = &if_state.condition;
        match ex {
            Expression::BinaryExpression(binex) => {
                if_quad.op = QuadrupelOp::from(binex.operator);
                if_quad.arg1 = self.eval_expression(&binex.left);
                if_quad.arg2 = self.eval_expression(&binex.right);
                if_quad.result = QuadrupelResult::Label(jmp_label.to_string());
                self.quadrupels.push(if_quad);
            }
            _ => panic!("mistake in 'if' expression!"),
        }
        self.eval_statement(&if_state.then_branch);
        self.add_label(Some(format!("L{}", jmp_label)));
        if let Some(state) = &if_state.else_branch {
            self.eval_statement(state);
        }
    }

    fn eval_while_statement(&mut self, while_state: &'a WhileStatement) {
        let jmp_label = self.label_num;
        self.label_num += 1;
        let while_label = self.label_num;
        self.label_num += 1;
        let mut while_quad = Quadrupel::new();
        self.add_label(Some(format!("L{}", while_label)));
        let ex = &while_state.condition;
        match ex {
            Expression::BinaryExpression(binex) => {
                while_quad.op = QuadrupelOp::from(binex.operator);
                while_quad.arg1 = self.eval_expression(&binex.left);
                while_quad.arg2 = self.eval_expression(&binex.right);
                while_quad.result = QuadrupelResult::Label(jmp_label.to_string());
                self.quadrupels.push(while_quad);
            }
            _ => panic!("mistake in 'while' expression!"),
        }
        self.eval_statement(&while_state.body);
        self.quadrupels.push(Quadrupel {
            op: QuadrupelOp::Goto,
            arg1: QuadrupelArg::Empty,
            arg2: QuadrupelArg::Empty,
            result: QuadrupelResult::Label(format!("L{}", while_label)),
        });
        self.add_label(Some(format!("L{}", jmp_label)));
    }

    fn eval_call_statement(&mut self, call_state: &'a CallStatement) {
        let mut count = 0;
        let name = call_state.name.clone();
        for param in &call_state.arguments {
            count += 1;
            let param = self.eval_expression(param);
            let mut quad = Quadrupel::new();
            quad.op = QuadrupelOp::Param;
            quad.arg1 = param;
            self.quadrupels.push(quad);
        }
        let mut quad = Quadrupel::new();
        quad.op = QuadrupelOp::Call;
        quad.arg1 = QuadrupelArg::Var(QuadrupelVar::Spl(name));
        quad.arg2 = QuadrupelArg::Const(count);
        self.quadrupels.push(quad);
    }

    fn eval_expression(&mut self, exp: &Expression) -> QuadrupelArg {
        match exp {
            Expression::BinaryExpression(exp) => {
                let left = self.eval_expression(&exp.left);
                let right = self.eval_expression(&exp.right);
                QuadrupelArg::Var(self.emit_expression_bin(exp.operator, left, right))
            }
            Expression::UnaryExpression(exp) => {
                let left = self.eval_expression(&exp.operand);
                QuadrupelArg::Var(self.emit_expression_un(exp.operator, left))
            }
            Expression::IntLiteral(val) => QuadrupelArg::Const(*val),
            Expression::VariableExpression(var) => self.eval_expression_var(var),
        }
    }

    fn eval_expression_var(&mut self, var: &Variable) -> QuadrupelArg {
        match var {
            Variable::NamedVariable(name) => QuadrupelArg::Var(QuadrupelVar::Spl(name.to_string())),
            Variable::ArrayAccess(array_access) => {
                let (var, offset) = self.eval_array_access(array_access);
                QuadrupelArg::Var(self.emit_expression_arr_acc(var, offset))
            }
        }
    }

    fn emit_expression_arr_acc(
        &mut self,
        array_var: QuadrupelVar,
        offset: QuadrupelVar,
    ) -> QuadrupelVar {
        let tmp = self.add_tmp_var();
        let mut quad = Quadrupel::new();
        quad.op = QuadrupelOp::ArrayLoad;
        quad.arg1 = QuadrupelArg::Var(array_var);
        quad.arg2 = QuadrupelArg::Var(offset);
        quad.result = QuadrupelResult::Var(tmp.clone());
        self.quadrupels.push(quad);
        tmp
    }

    fn emit_expression_bin(
        &mut self,
        op: Operator,
        left: QuadrupelArg,
        right: QuadrupelArg,
    ) -> QuadrupelVar {
        let tmp = self.add_tmp_var();
        let mut quad = Quadrupel::new();
        quad.op = op.into();
        quad.arg1 = left;
        quad.arg2 = right;
        quad.result = QuadrupelResult::Var(tmp.clone());
        self.quadrupels.push(quad);
        tmp
    }

    fn emit_expression_un(&mut self, op: UnaryOperator, left: QuadrupelArg) -> QuadrupelVar {
        let tmp = self.add_tmp_var();
        let mut quad = Quadrupel::new();
        quad.op = op.into();
        quad.arg1 = left;
        quad.result = QuadrupelResult::Var(tmp.clone());
        self.quadrupels.push(quad);
        tmp
    }

    fn get_base_name(&mut self, variable: &'a Variable, rname: String) -> String {
        match variable {
            Variable::NamedVariable(name) => format!("{}{}", name.clone(), rname),
            Variable::ArrayAccess(array_access) => {
                let index_value = self.eval_expression(&array_access.index.clone());
                let new_rname = format!("{}[{}]", rname, index_value);
                self.get_base_name(&array_access.array, new_rname)
            }
        }
    }

    fn add_label(&mut self, name: Option<String>) {
        let label: String;
        if let Some(name) = name {
            label = name;
        } else {
            label = format!("L{}", self.label_num);
            self.label_num += 1;
        }
        let mut new_quad: Quadrupel = Quadrupel::new();
        new_quad.result = QuadrupelResult::Label(label);
        self.quadrupels.push(new_quad);
    }

    fn add_tmp_var(&mut self) -> QuadrupelVar {
        let n = self.temp_var_count;
        self.temp_var_count += 1;

        QuadrupelVar::Tmp(n)
    }
}

impl Quadrupel {
    fn new() -> Self {
        Quadrupel {
            op: super::QuadrupelOp::Default,
            arg1: QuadrupelArg::Empty,
            arg2: QuadrupelArg::Empty,
            result: QuadrupelResult::Empty,
        }
    }
}
