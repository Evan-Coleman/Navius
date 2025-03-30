# Design Evaluation Report: navius-cli

**Date:** March 29, 2025  
**Evaluator:** Workspace Migration Team  
**Crate Version:** 0.1.0 (Proposed)

## Overview

The `navius-cli` crate is intended to provide command-line interface tools for the Navius framework. This evaluation outlines a proposed architecture, interfaces, and implementation patterns for the CLI crate based on existing documentation and code references. The goal is to create a comprehensive, user-friendly CLI that follows modern Rust CLI development patterns and integrates well with the overall Navius ecosystem.

## Proposed Architecture

Based on the existing examples and documentation, we propose the following architecture for the `navius-cli` crate:

1. **Core Command Structure**: A well-organized command hierarchy using the Clap framework for argument parsing and command organization.

2. **Feature Management**: Tools for enabling, disabling, and managing framework features.

3. **Project Management**: Tools for creating, building, and managing Navius projects.

4. **Development Utilities**: Tools for database migrations, code generation, and other development tasks.

5. **Integration with Navius Components**: Direct integration with other Navius crates for tasks like dependency injection configuration, plugin management, and template rendering.

6. **Modern CLI Experience**: Integration with libraries like Dialoguer, Indicatif, and colored for a rich, interactive CLI experience.

The architecture enables:
- Modular addition of new commands and features
- Consistent user experience across different CLI functionalities
- Easy integration with Navius framework components
- Rich interactive interfaces with visual feedback
- Comprehensive error handling and user guidance

## Proposed Interfaces

### Primary Traits and Interfaces

1. **CommandProvider** Trait:
   ```rust
   pub trait CommandProvider {
       fn command_name(&self) -> &'static str;
       fn command_description(&self) -> &'static str;
       fn build_command(&self) -> Command;
       fn execute(&self, matches: &ArgMatches) -> Result<(), CliError>;
   }
   ```

2. **CommandRegistry** Struct:
   ```rust
   pub struct CommandRegistry {
       commands: HashMap<String, Box<dyn CommandProvider>>,
   }
   
   impl CommandRegistry {
       pub fn new() -> Self;
       pub fn register_command(&mut self, command: Box<dyn CommandProvider>);
       pub fn build_cli(&self) -> Command;
       pub fn execute_command(&self, command_name: &str, matches: &ArgMatches) 
           -> Result<(), CliError>;
   }
   ```

3. **CliConfig** Struct:
   ```rust
   pub struct CliConfig {
       config_path: PathBuf,
       output_format: OutputFormat,
       verbose: bool,
       quiet: bool,
       color: ColorChoice,
   }
   ```

4. **CliContext** Struct for sharing data between commands:
   ```rust
   pub struct CliContext {
       config: CliConfig,
       workspace_path: PathBuf,
       registry: Option<CommandRegistry>,
       // ... other shared state
   }
   ```

### Error Handling

The CLI should use a dedicated error type that provides user-friendly error messages:

```rust
pub enum CliError {
    ConfigError(String),
    CommandError(String),
    InvalidInput(String),
    ExecutionError(String),
    IoError(String),
    // ... additional error variants
}
```

## Proposed Implementation Structure

### Directory Structure

```
navius-cli/
├── Cargo.toml
├── src/
│   ├── lib.rs                 # Library exports
│   ├── bin/
│   │   └── navius.rs          # Main entry point
│   ├── commands/              # Command implementations
│   │   ├── mod.rs             # Command module exports
│   │   ├── feature.rs         # Feature management commands
│   │   ├── project.rs         # Project management commands
│   │   ├── database.rs        # Database management commands
│   │   ├── generate.rs        # Code generation commands
│   │   ├── plugin.rs          # Plugin management commands
│   │   └── template.rs        # Template commands
│   ├── config/                # Configuration handling
│   │   ├── mod.rs
│   │   └── cli_config.rs
│   ├── ui/                    # User interface utilities
│   │   ├── mod.rs
│   │   ├── progress.rs        # Progress indicators
│   │   ├── prompts.rs         # User prompts
│   │   └── output.rs          # Formatted output
│   ├── utils/                 # Utility functions
│   │   ├── mod.rs
│   │   ├── fs.rs              # File system utilities
│   │   └── process.rs         # Process execution utilities
│   ├── context.rs             # CLI context for command execution
│   ├── error.rs               # Error types and handling
│   └── registry.rs            # Command registry implementation
├── templates/                 # CLI template files
└── tests/                     # Integration tests
```

### Implementation Patterns

#### Plugin Architecture for Commands

The CLI should use a plugin-based architecture where commands are dynamically registered with the command registry:

```rust
fn main() {
    let mut registry = CommandRegistry::new();
    
    // Register built-in commands
    registry.register_command(Box::new(FeatureCommand::new()));
    registry.register_command(Box::new(ProjectCommand::new()));
    registry.register_command(Box::new(DatabaseCommand::new()));
    // ... register additional commands
    
    // Build and run the CLI
    let cli = registry.build_cli();
    let matches = cli.get_matches();
    
    if let Err(error) = registry.execute_matches(&matches) {
        // Handle error and exit with appropriate exit code
    }
}
```

#### Builder Pattern for Commands

Each command should use the builder pattern for its configuration, providing a consistent and readable way to define command options:

```rust
impl FeatureCommand {
    pub fn new() -> Self {
        Self::default()
    }
    
    fn build_command(&self) -> Command {
        Command::new(self.command_name())
            .about(self.command_description())
            .subcommand(self.build_list_subcommand())
            .subcommand(self.build_enable_subcommand())
            .subcommand(self.build_disable_subcommand())
            // ... additional subcommands
    }
    
    fn build_list_subcommand(&self) -> Command {
        Command::new("list")
            .about("List available features")
            .arg(
                Arg::new("format")
                    .short('f')
                    .long("format")
                    .help("Output format (text, json, yaml)")
                    .default_value("text")
                    .value_parser(["text", "json", "yaml"]),
            )
    }
    
    // ... additional subcommand builders
}
```

#### Context Sharing Between Commands

Commands should use a shared context for accessing configuration and shared data:

```rust
impl CommandProvider for FeatureCommand {
    fn execute(&self, context: &mut CliContext, matches: &ArgMatches) -> Result<(), CliError> {
        match matches.subcommand() {
            Some(("list", sub_matches)) => self.list_features(context, sub_matches),
            Some(("enable", sub_matches)) => self.enable_feature(context, sub_matches),
            Some(("disable", sub_matches)) => self.disable_feature(context, sub_matches),
            // ... handle other subcommands
            _ => Err(CliError::CommandError("Invalid subcommand".to_string())),
        }
    }
}
```

#### Rich Terminal UI

The CLI should use modern terminal libraries for a rich user experience:

```rust
fn interactive_feature_selection(context: &CliContext) -> Result<Vec<String>, CliError> {
    let items = context.get_feature_registry().get_feature_list();
    let selections = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select features to enable")
        .items(&items)
        .interact()
        .map_err(|e| CliError::IoError(format!("Failed to display selection: {}", e)))?;
        
    // Process selections and return selected feature names
    // ...
}
```

#### Integration with Navius Framework Components

The CLI should directly integrate with other Navius crates:

```rust
fn create_project(context: &mut CliContext, project_name: &str) -> Result<(), CliError> {
    // Create project directory structure
    // ...
    
    // Initialize template engine
    let template_registry = navius_template::TemplateEngineRegistryBuilder::new()
        .with_engine(Box::new(navius_template::HandlebarsTemplateEngineFactory::new()))
        .build()?;
    
    // Render project templates
    // ...
    
    // Initialize dependency injection
    let di_config = navius_di::AppConfigBuilder::new()
        .with_component_scan_path("src")
        .build()?;
    
    // ...
}
```

## Key Components

### 1. Feature Management

Commands for managing Navius framework features:

- `navius feature list`: List available features
- `navius feature enable <name>`: Enable a feature
- `navius feature disable <name>`: Disable a feature
- `navius feature status`: Show current feature status
- `navius feature interactive`: Interactive feature selection
- `navius feature analyze-deps`: Analyze feature dependencies

### 2. Project Management

Commands for creating and managing Navius projects:

- `navius new <name>`: Create a new project
- `navius build`: Build the project
- `navius run`: Run the project
- `navius test`: Run tests
- `navius check`: Validate project structure

### 3. Database Management

Commands for database operations:

- `navius db migrate`: Run database migrations
- `navius db reset`: Reset the database
- `navius db seed`: Seed the database with sample data
- `navius db status`: Show migration status

### 4. Code Generation

Commands for generating code:

- `navius generate controller <name>`: Generate a controller
- `navius generate model <name>`: Generate a model
- `navius generate migration <name>`: Generate a migration
- `navius generate service <name>`: Generate a service

### 5. Plugin Management

Commands for managing plugins:

- `navius plugin list`: List available plugins
- `navius plugin install <name>`: Install a plugin
- `navius plugin remove <name>`: Remove a plugin
- `navius plugin create <name>`: Create a new plugin

### 6. Template Management

Commands for managing templates:

- `navius template list`: List available templates
- `navius template render <name>`: Render a template
- `navius template install <name>`: Install a template
- `navius template create <name>`: Create a new template

## Integration with Navius Ecosystem

The CLI should integrate with the broader Navius ecosystem:

1. **navius-core**: Use core utilities, configuration, and logging
2. **navius-di**: Integration with dependency injection for project setup
3. **navius-template**: Template rendering for code generation
4. **navius-plugin**: Plugin management for extensibility
5. **navius-db**: Database operations for migration commands

## Recommendations

Based on the analysis, we recommend the following for the implementation of the `navius-cli` crate:

1. **Command Structure**: Use Clap for argument parsing with a modular structure for easy extension.
2. **Plugin Architecture**: Implement a plugin system for commands to allow for extensibility.
3. **Rich Terminal UI**: Use Dialoguer, Indicatif, and Colored for an interactive, visually appealing UI.
4. **Integration with Framework**: Make the CLI a central part of the Navius developer experience by integrating with core framework components.
5. **Comprehensive Error Handling**: Provide clear, actionable error messages to guide users.
6. **Extensive Documentation**: Include comprehensive help text, examples, and error guidance.
7. **Good Default Behaviors**: Ensure the CLI works well with minimal configuration while allowing customization.
8. **Testing Infrastructure**: Include both unit tests and integration tests that verify command behavior.
9. **Configuration Management**: Support multiple levels of configuration (project, user, system).
10. **Shell Completions**: Generate shell completions for better command-line integration.

## Conclusion

The proposed `navius-cli` crate will provide a comprehensive command-line interface for the Navius framework, focusing on developer experience and integration with the broader ecosystem. By following modern CLI design patterns and leveraging the Rust ecosystem's best CLI libraries, the crate will offer an intuitive, powerful interface for Navius developers.

The modular architecture with a plugin system for commands will allow for easy extension and customization, while the rich terminal UI will provide a modern, interactive experience. Integration with other Navius components ensures that the CLI is a central part of the framework's developer experience.

Implementing this design will require careful attention to user experience, error handling, and documentation, but the result will be a powerful tool that significantly enhances the Navius development workflow. 