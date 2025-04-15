#![expect(unused, unused_variables)]
use std::fmt::format;

use crate::absyn::{
    absyn::Statement, assign_statement::AssignStatement, call_statement::CallStatement,
    if_statement::IfStatement, procedure_definition::ProcedureDefinition,
    while_statement::WhileStatement,
};

use super::{Quadrupel, QuadrupelOp, Tac};

impl<'a> Tac<'a> {
    pub(super) fn eval_proc_def(&mut self, proc_def: &ProcedureDefinition) {
        let mut new_quad: Quadrupel = Quadrupel::new();
        new_quad.op = QuadrupelOp::Label(proc_def.name.clone());
        self.quadrupels.push(new_quad);
        for statement in &proc_def.body {
            self.eval_statement(statement);
        }
    }

    fn eval_statement(&self, statement: &Statement) {
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

    fn eval_assign_statement(&self, assign: &AssignStatement) {}

    fn eval_if_statement(&self, if_state: &IfStatement) {}

    fn eval_while_statement(&self, while_state: &WhileStatement) {}

    fn eval_call_statement(&self, call_state: &CallStatement) {}

    fn eval_expression(&self) {}

    fn add_label(&mut self, name: Option<String>) {
        self.label_stack.push(self.label_num);
        let label: String;
        if let Some(name) = name {
            label = name;
        } else {
            label = format!("T{}:", self.label_num);
        }
        self.label_table.insert(label.clone(), self.label_num);
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
