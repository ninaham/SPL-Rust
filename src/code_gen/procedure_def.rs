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
    pub(super) fn eval_proc_def(&mut self, proc_def: &ProcedureDefinition) {
        let mut new_quad: Quadrupel = Quadrupel::new();
        new_quad.op = QuadrupelOp::Label(proc_def.name.clone());
        self.quadrupels.push(new_quad);
        for statement in &proc_def.body {
            self.eval_statement(statement);
        }
    }

    fn eval_statement(&mut self, statement: &Statement) {
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

    fn eval_assign_statement(&mut self, assign: &AssignStatement) {
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

    fn eval_if_statement(&mut self, if_state: &IfStatement) {}

    fn eval_while_statement(&mut self, while_state: &WhileStatement) {}

    fn eval_call_statement(&self, call_state: &CallStatement) {}

    fn eval_expression(&self, exp: &Expression) -> String {
        "".to_string()
    }

    fn get_base_name(&self, variable: &'a Variable, rname: String) -> String {
        match variable {
            Variable::NamedVariable(name) => format!("{}{}", name, rname),
            Variable::ArrayAccess(array_access) => self.get_base_name(
                &array_access.array,
                format!("{}[{}]", rname, self.eval_expression(&array_access.index)),
            ),
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
