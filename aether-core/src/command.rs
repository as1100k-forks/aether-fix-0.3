use crate::config::{Bot, Role};

pub enum AetherCommand {
    PearlLoad,
    Help,
    Unknown,
}

impl AetherCommand {
    // pub fn parse(input: &str, state: &Bot) -> Self {
    //     let parts: Vec<&str> = input.split_whitespace().collect();
    //
    //     // TODO: Port all the commands to discord and only support necessary ones via minecraft chat
    //     match parts.as_slice() {
    //         ["!pearl", "load"] => {
    //             if state.role == Role::Pearl {
    //                 AetherCommand::PearlLoad
    //             } else {
    //                 AetherCommand::Unknown
    //             }
    //         }
    //         ["!help"] => AetherCommand::Help,
    //         _ => AetherCommand::Unknown,
    //     }
    // }

    // This is a temporary solution as azalea gives message twice
    // and this way we can ignore the words after
    pub fn parse(input: &str, state: &Bot) -> Self {
        let mut parts = input.split_whitespace();

        // Take the first word as the command
        match parts.next() {
            Some("!pearl") => {
                match parts.next() {
                    Some("load") => {
                        if state.role == Role::Pearl {
                            AetherCommand::PearlLoad
                        } else {
                            AetherCommand::Unknown
                        }
                    }
                    _ => AetherCommand::Unknown,
                }
            }
            Some("!help") => AetherCommand::Help,
            _ => AetherCommand::Unknown,
        }
    }
}
