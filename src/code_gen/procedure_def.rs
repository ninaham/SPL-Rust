#![expect(unused, unused_variables)]
use std::fmt::format;

use crate::absyn::{
    absyn::{Expression, Statement, Variable},
    assign_statement::{self, AssignStatement},
    call_statement::CallStatement,
    if_statement::IfStatement,
    procedure_definition::ProcedureDefinition,
    while_statement::WhileStatement,
};

use super::{Quadrupel, QuadrupelOp, Tac};

impl<'a> Tac<'a> {
    pub(super) fn eval_proc_def(&mut self, proc_def: &'a ProcedureDefinition) {
        let mut new_quad: Quadrupel = Quadrupel::new();
        new_quad.op = QuadrupelOp::Label(proc_def.name.clone());
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
        if let Variable::NamedVariable(name) = &assign.target {
            assign_quad.op = QuadrupelOp::Assign;
            assign_quad.arg1 = name.clone();
        } else {
            assign_quad.op = QuadrupelOp::ArrayStore;
            assign_quad.arg1 = self.get_base_name(&assign.target, "".to_string())
        }

        assign_quad.arg2 = self.eval_expression(&assign.value);
        self.quadrupels.push(assign_quad);
    }

    fn eval_if_statement(&mut self, if_state: &'a IfStatement) {
        todo!("fill me with code")
    }

    fn eval_while_statement(&mut self, while_state: &WhileStatement) {
        todo!("fill me with code")
    }

    fn eval_call_statement(&mut self, call_state: &'a CallStatement) {
        let mut count = 0;
        let name = call_state.name.clone();
        for param in &call_state.arguments {
            count += 1;
            let param = self.eval_expression(&param);
            let mut quad = Quadrupel::new();
            quad.op = QuadrupelOp::Param;
            quad.arg1 = param;
            self.quadrupels.push(quad);
        }
        let mut quad = Quadrupel::new();
        quad.op = QuadrupelOp::Call;
        quad.arg1 = name;
        quad.arg2 = count.to_string();
        self.quadrupels.push(quad);
    }

    fn eval_expression(&mut self, exp: &Expression) -> String {
        todo!("fill me with code")
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
        self.label_stack.push(self.label_num);
        let label: String;
        if let Some(name) = name {
            label = name;
        } else {
            label = format!("T{}:", self.label_num);
        }
        self.label_num += 1;
        let mut new_quad: Quadrupel = Quadrupel::new();
        new_quad.op = QuadrupelOp::Label(label);
        self.quadrupels.push(new_quad);
    }
}

impl Quadrupel {
    fn new() -> Self {
        Quadrupel {
            op: super::QuadrupelOp::Default,
            arg1: "".to_string(),
            arg2: "".to_string(),
            result: "".to_string(),
        }
    }
}
