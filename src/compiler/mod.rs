use crate::Action;
use std::collections::HashMap;
use structopt::StructOpt;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Command {
    CursorRight(u32),
    CursorLeft(u32),
    Add(u32),
    Subtract(u32),
    Get,
    ToSx,
    FromSx,
    ToTx,
    FromTx,
    ToAx,
    FromAx,
    Heat,
    Transfer,
    NoOp
}

#[derive(StructOpt, Debug)]
pub struct CompilerFlags {
    #[structopt(short, long)]
    sideproduct_pills:bool
}

const ZERO:u32 = 0;
const MAKE_PILL:u32 = 11;
const MAKE_VIAL:u32 = 12;
const EJECT:u32 = 13;
const ALL:u32 = 100;

/// constants that will appear even if they don't have a reference in an action
static FORCED_CONSTANTS: &'static [u32] = &[ZERO, MAKE_PILL, MAKE_VIAL, EJECT, ALL];

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProgramState {
    pointer_position:u32,
    constants:HashMap<u32,u32>,
    scratch:u32
}

impl ProgramState {
    fn move_cursor_right(&mut self, amount:u32) -> Command {
        self.pointer_position += amount;
        return Command::CursorRight(amount);
    }

    fn move_cursor_left(&mut self, amount:u32) -> Command {
        self.pointer_position -= amount;
        return Command::CursorLeft(amount);
    }

    fn goto_register(&mut self, register:u32) -> Command {
        if register == self.pointer_position {
            return Command::NoOp
        } else if register > self.pointer_position {
            return self.move_cursor_right(register - self.pointer_position);
        } else {
            return self.move_cursor_left(self.pointer_position - register);
        }
    }

    fn goto_constant(&mut self, constant:u32) -> Command {
        if !self.constants.contains_key(&constant) {
            panic!("MISSING: {}", constant);
        }
        let register = *self.constants.get(&constant).unwrap();
        return self.goto_register(register);
    }
}

pub fn compile (actions:&Vec<Action>, flags:&CompilerFlags) -> Vec<Command> {
    let (constants, scratch) = extract_constants(&actions);
    let pointer_position = 0;
    let mut state = ProgramState{constants, pointer_position, scratch:scratch};
    let mut commands = vec![];
    create_constants(&mut state, &mut commands);
    for action in actions {
        create_commands_from_action(&mut commands, &mut state, action, &flags);
    }
    return commands;
}

fn create_constants (state:&mut ProgramState, commands:&mut Vec<Command>) {
    let mut constants:Vec<u32> = state.constants.keys().map(|x| *x).collect();
    constants.sort_by(|x1,x2| state.constants.get(x1).unwrap().cmp(state.constants.get(x2).unwrap()));
    for constant in constants {
        commands.push(state.goto_register(*state.constants.get(&constant).unwrap()).clone());
        commands.push(Command::Add(constant))
    }
}

fn extract_constants (actions:&Vec<Action>) -> (HashMap<u32,u32>, u32) {
    let mut map = HashMap::new();
    let mut register_counter = 0;
    for action in actions {
        match *action {
            Action::Transfer{amount, source, target} => {
                add_constant(&mut map, amount, &mut register_counter);
                add_constant(&mut map, source, &mut register_counter);
                add_constant(&mut map, target, &mut register_counter);
            },
            Action::Heat{temp, target} => {
                if temp > 273 {
                    add_constant(&mut map, temp-273, &mut register_counter);
                } else {
                    add_constant(&mut map, 273-temp, &mut register_counter);
                }
                add_constant(&mut map, target, &mut register_counter);
            },
            Action::Eject{target} => {
                add_constant(&mut map, target, &mut register_counter);
            },
            Action::DumpByproduct{target, remaining} => {
                add_constant(&mut map, target, &mut register_counter);
                add_constant(&mut map, remaining, &mut register_counter);
            },
            Action::EjectDownTo{target, amount} => {
                add_constant(&mut map, target, &mut register_counter);
                add_constant(&mut map, amount, &mut register_counter);
            },
            Action::CreateBottle{target, amount} => {
                add_constant(&mut map, target, &mut register_counter);
                add_constant(&mut map, amount, &mut register_counter);
            },
            Action::CreatePill{target, amount} => {
                add_constant(&mut map, target, &mut register_counter);
                add_constant(&mut map, amount, &mut register_counter);
            }
        }
    }
    for x in FORCED_CONSTANTS {
        add_constant(&mut map, *x, &mut register_counter);
    }

    println!("{:?}", &map);
    return (map, register_counter);
}

fn create_commands_from_action (commands:&mut Vec<Command>, state:&mut ProgramState, action:&Action, flags:&CompilerFlags) {
    match *action {
        Action::Transfer{amount, source, target} => {
            commands.push(state.goto_constant(amount));
            commands.push(Command::ToAx);
            commands.push(state.goto_constant(source));
            commands.push(Command::ToSx);
            commands.push(state.goto_constant(target));
            commands.push(Command::ToTx);
            commands.push(Command::Transfer);
        },
        Action::Heat{temp, target} => {
            if temp > 273 {
                commands.push(state.goto_constant(temp - 273));
                commands.push(Command::ToAx);
                commands.push(state.goto_constant(ZERO));
                commands.push(Command::ToTx);
            } else {
                commands.push(state.goto_constant(ZERO));
                commands.push(Command::ToAx);
                commands.push(state.goto_constant(273-temp));
                commands.push(Command::ToTx);

            }
            commands.push(state.goto_constant(target));
            commands.push(Command::ToSx);
            commands.push(Command::Heat);
        },
        Action::Eject{target} => {
            commands.push(state.goto_constant(target));
            commands.push(Command::ToSx);
            commands.push(state.goto_constant(ALL));
            commands.push(Command::ToAx);
            commands.push(state.goto_constant(MAKE_PILL));
            commands.push(Command::ToTx);
            commands.push(Command::Transfer);
        },
        Action::DumpByproduct{target, remaining} => {
            commands.push(state.goto_constant(target));
            commands.push(Command::ToSx);
            commands.push(Command::Get);
            commands.push(state.goto_register(state.scratch));
            commands.push(Command::FromAx);
            commands.push(Command::Subtract(remaining));
            commands.push(Command::ToAx);
            if flags.sideproduct_pills {
                commands.push(state.goto_constant(MAKE_PILL));
            } else {
                commands.push(state.goto_constant(EJECT));
            }
            commands.push(Command::ToTx);
            commands.push(Command::Transfer);
        },
        Action::EjectDownTo{target, amount} => {
            commands.push(state.goto_constant(target));
            commands.push(Command::ToSx);
            commands.push(Command::Get);
            commands.push(state.goto_register(state.scratch));
            commands.push(Command::FromAx);
            commands.push(Command::Subtract(amount));
            commands.push(Command::ToAx);
            if flags.sideproduct_pills {
                commands.push(state.goto_constant(MAKE_PILL));
            } else {
                commands.push(state.goto_constant(EJECT));
            }
            commands.push(Command::ToTx);
            commands.push(Command::Transfer);
        },
        Action::CreateBottle{target, amount} => {
            commands.push(state.goto_constant(target));
            commands.push(Command::ToSx);
            commands.push(state.goto_constant(amount));
            commands.push(Command::ToAx);
            commands.push(state.goto_constant(MAKE_VIAL));
            commands.push(Command::ToTx);
            commands.push(Command::Transfer);
        },
        Action::CreatePill{target, amount} => {
            commands.push(state.goto_constant(target));
            commands.push(Command::ToSx);
            commands.push(state.goto_constant(amount));
            commands.push(Command::ToAx);
            commands.push(state.goto_constant(MAKE_PILL));
            commands.push(Command::ToTx);
            commands.push(Command::Transfer);
        },
    }
}

fn add_constant(map:&mut HashMap<u32,u32>, constant:u32, counter:&mut u32) {
    if !map.contains_key(&constant) {
        map.insert(constant, *counter);
        *counter += 1;
    }
}


pub fn to_bytecode (commands:&Vec<Command>) -> String {
    let mut code = "".to_string();
    for command in commands {
        let command_code = match *command {
            Command::CursorRight(amount) => {
                (0..amount).map(|_| ">").collect::<String>()
            },
            Command::CursorLeft(amount) => {
                (0..amount).map(|_| "<").collect::<String>()
            },
            Command::Add(amount) => {
                (0..amount).map(|_| "+").collect::<String>()
            },
            Command::Subtract(amount) => {
                (0..amount).map(|_| "-").collect::<String>()
            },
            Command::Get => {
                ",".to_string()
            },
            Command::ToSx => {
                "}".to_string()
            },
            Command::FromSx => {
                "{".to_string()
            },
            Command::ToTx => {
                ")".to_string()
            },
            Command::FromTx => {
                "(".to_string()
            },
            Command::ToAx => {
                "'".to_string()
            },
            Command::FromAx => {
                "^".to_string()
            },
            Command::Heat => {
                "$".to_string()
            },
            Command::Transfer => {
                "@".to_string()
            },
            Command::NoOp => {
                "".to_string()
            }
        };
        code = format!("{}{}",code,command_code);
    }
    return format!("{}~",code);
}
