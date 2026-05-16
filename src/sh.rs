use crate::errors::*;
use iocore::Path;

pub fn shell_command(command: impl std::fmt::Display, current_dir: impl Into<Path>) -> Result<i32> {
    let command = command.to_string();
    eprintln!("running {:#?}", &command);
    match iocore::shell_command(&command, current_dir) {
        Ok(exit_code) => {
            if exit_code != 0 {
                Err(Error::CliError(format!(
                    "command {:#?} failed with exit code {}",
                    &command, exit_code
                )))
            } else {
                Ok(exit_code)
            }
        }
        Err(error) => Err(Error::CliError(format!(
            "command {:#?} failed with error: {}",
            &command, error
        ))),
    }
}
