use super::super::grammar::*;
use std::collections::HashMap;

type SymbolsTable = HashMap<String, SymbolEntry>;

#[derive(Debug, PartialEq)]
pub struct ClassSymbols {
    pub entries: SymbolsTable,
    field_counter: u16,
    static_counter: u16,
}

impl ClassSymbols {
    pub fn new(class_var_decs: &Vec<ClassVarDec>) -> Self {
        let mut field_counter: u16 = 0;
        let mut static_counter: u16 = 0;
        let mut entries = SymbolsTable::new();

        class_var_decs.iter().for_each(|var| {
            let entry = match var.var_type {
                ClassVarType::Field => {
                    field_counter += 1;
                    SymbolEntry {
                        index: field_counter - 1,
                        jtype: var.jack_type.clone(),
                        kind: Kind::Field,
                    }
                }
                ClassVarType::Static => {
                    static_counter += 1;
                    SymbolEntry {
                        index: static_counter - 1,
                        jtype: var.jack_type.clone(),
                        kind: Kind::Static,
                    }
                }
            };

            entries.insert(var.name.clone(), entry);
        });

        Self {
            entries,
            field_counter,
            static_counter,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct RoutineSymbols {
    pub entries: SymbolsTable,
    local_counter: u16,
    argument_counter: u16,
}

impl RoutineSymbols {
    pub fn new(routine: &SubroutineDec, class_name: &String) -> Self {
        let (entries, argument_counter) = match routine.routine_type {
            RoutineType::Method => (
                SymbolsTable::from([(
                    "this".into(),
                    SymbolEntry {
                        jtype: JackType::Class(class_name.into()),
                        kind: Kind::Argument,
                        index: 0,
                    },
                )]),
                1,
            ),
            _ => (SymbolsTable::new(), 0),
        };

        let mut symbols = Self {
            entries: entries,
            local_counter: 0,
            argument_counter: argument_counter,
        };

        routine
            .parameters
            .iter()
            .for_each(|param| symbols.add_param(param));

        routine
            .body
            .vars
            .iter()
            .for_each(|var| symbols.add_var(var));

        symbols
    }

    fn add_param(self: &mut Self, param: &Parameter) {
        let entry = SymbolEntry {
            kind: Kind::Argument,
            jtype: param.jack_type.clone(),
            index: self.argument_counter,
        };

        self.entries.insert(param.name.clone(), entry);
        self.argument_counter += 1;
    }

    fn add_var(self: &mut Self, var: &VarDec) {
        let entry = SymbolEntry {
            kind: Kind::Local,
            jtype: var.jack_type.clone(),
            index: self.local_counter,
        };

        self.entries.insert(var.name.clone(), entry);
        self.local_counter += 1;
    }
}

#[derive(Debug, PartialEq)]
pub struct SymbolEntry {
    pub kind: Kind,
    pub jtype: JackType,
    pub index: u16,
}

#[derive(Debug, PartialEq)]
pub enum Kind {
    Local,
    Argument,
    Field,
    Static,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handle_class_symbol_tables() {
        let class = Class {
            name: "Example".into(),
            vars: vec![ClassVarDec {
                var_type: ClassVarType::Static,
                jack_type: JackType::Int,
                name: "a".into(),
            }],
            routines: vec![],
        };

        let expected_class_symbols = ClassSymbols {
            entries: SymbolsTable::from([(
                "a".into(),
                SymbolEntry {
                    jtype: JackType::Int,
                    kind: Kind::Static,
                    index: 0,
                },
            )]),
            field_counter: 0,
            static_counter: 1,
        };

        let actual_class_symbols = ClassSymbols::new(&class.vars);
        assert_eq!(expected_class_symbols, actual_class_symbols);
    }

    #[test]
    fn handle_routine_symbol_tables() {
        let class = Class {
            name: "Example".into(),
            vars: vec![],
            routines: vec![SubroutineDec {
                routine_type: RoutineType::Method,
                return_type: ReturnType::Void,
                name: "first".into(),
                parameters: vec![
                    Parameter {
                        name: "b".into(),
                        jack_type: JackType::Char,
                    },
                    Parameter {
                        name: "c".into(),
                        jack_type: JackType::Char,
                    },
                ],
                body: SubroutineBody {
                    vars: vec![VarDec {
                        jack_type: JackType::Boolean,
                        name: "d".into(),
                    }],
                    statements: vec![Statement::Return(None)],
                },
            }],
        };

        let expected_routine_symbols = RoutineSymbols {
            entries: SymbolsTable::from([
                (
                    "this".into(),
                    SymbolEntry {
                        jtype: JackType::Class("Example".into()),
                        kind: Kind::Argument,
                        index: 0,
                    },
                ),
                (
                    "b".into(),
                    SymbolEntry {
                        jtype: JackType::Char,
                        kind: Kind::Argument,
                        index: 1,
                    },
                ),
                (
                    "c".into(),
                    SymbolEntry {
                        jtype: JackType::Char,
                        kind: Kind::Argument,
                        index: 2,
                    },
                ),
                (
                    "d".into(),
                    SymbolEntry {
                        jtype: JackType::Boolean,
                        kind: Kind::Local,
                        index: 0,
                    },
                ),
            ]),
            local_counter: 1,
            argument_counter: 3,
        };

        let actual_routine_symbols =
            RoutineSymbols::new(class.routines.first().unwrap(), &class.name);

        assert_eq!(expected_routine_symbols, actual_routine_symbols);
    }
}
