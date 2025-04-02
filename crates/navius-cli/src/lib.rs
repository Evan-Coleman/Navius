/*!
 * Navius CLI tools crate
 *
 * This crate provides command-line interface tools for the Navius framework.
 */

use thiserror::Error;

/// CLI-specific error types
#[derive(Debug, Error)]
pub enum CliError {
    /// Command execution error
    #[error("Command execution error: {0}")]
    CommandError(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Invalid input error
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

/// Result type for CLI operations
pub type CliResult<T> = Result<T, CliError>;

/// CLI command trait for implementing CLI commands
pub trait Command {
    /// Execute the command with the given arguments
    fn execute(&self, args: &[String]) -> CliResult<()>;

    /// Get the name of the command
    fn name(&self) -> &'static str;

    /// Get the description of the command
    fn description(&self) -> &'static str;
}

/// CLI command runner
pub struct CommandRunner {
    commands: Vec<Box<dyn Command>>,
}

impl CommandRunner {
    /// Create a new command runner
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    /// Register a command with the runner
    pub fn register_command<C: Command + 'static>(&mut self, command: C) {
        self.commands.push(Box::new(command));
    }

    /// Run a command by name with the given arguments
    pub fn run_command(&self, name: &str, args: &[String]) -> CliResult<()> {
        for cmd in &self.commands {
            if cmd.name() == name {
                return cmd.execute(args);
            }
        }

        Err(CliError::InvalidInput(format!("Unknown command: {}", name)))
    }

    /// List all available commands
    pub fn list_commands(&self) -> Vec<(&'static str, &'static str)> {
        self.commands
            .iter()
            .map(|cmd| (cmd.name(), cmd.description()))
            .collect()
    }
}

impl Default for CommandRunner {
    fn default() -> Self {
        Self::new()
    }
}

/// Utility function to parse command line arguments
pub fn parse_args(args: &[String]) -> CliResult<(String, Vec<String>)> {
    if args.is_empty() {
        return Err(CliError::InvalidInput("No command specified".to_string()));
    }

    let command = args[0].clone();
    let command_args = args[1..].to_vec();

    Ok((command, command_args))
}
