use std::marker::PhantomData;

/// State: Builder has no fields set
pub struct BuilderEmpty;

/// State: Builder has name set
pub struct BuilderWithName;

/// State: Builder has all required fields
pub struct BuilderComplete;

/// A configuration builder that uses typestate to ensure required fields are set
///
/// INVARIANT: A Config can only be built when all required fields are provided.
/// This is enforced at compile time through state transitions.
pub struct Builder<S> {
    name: Option<String>,
    description: Option<String>,
    timeout: Option<u64>,
    _state: PhantomData<S>,
}

impl Builder<BuilderEmpty> {
    /// Create a new empty builder
    pub fn new() -> Self {
        Builder {
            name: None,
            description: None,
            timeout: None,
            _state: PhantomData,
        }
    }

    /// Set the name (required field)
    ///
    /// INVARIANT: Name must be set before build() can be called.
    /// This transitions to BuilderWithName state.
    pub fn name(mut self, name: impl Into<String>) -> Builder<BuilderWithName> {
        self.name = Some(name.into());
        Builder {
            name: self.name,
            description: self.description,
            timeout: self.timeout,
            _state: PhantomData,
        }
    }
}

impl Builder<BuilderWithName> {
    /// Set the description (optional)
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the timeout (optional)
    pub fn timeout(mut self, timeout: u64) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Mark as complete (all required fields set)
    ///
    /// INVARIANT: This transition is only available after name is set
    pub fn complete(self) -> Builder<BuilderComplete> {
        Builder {
            name: self.name,
            description: self.description,
            timeout: self.timeout,
            _state: PhantomData,
        }
    }
}

impl Builder<BuilderComplete> {
    /// Build the final configuration
    ///
    /// INVARIANT: Can only build when in BuilderComplete state,
    /// which guarantees all required fields are set.
    pub fn build(self) -> Config {
        Config {
            name: self.name.expect("Name should be set in BuilderComplete state"),
            description: self.description.unwrap_or_else(|| "No description".to_string()),
            timeout: self.timeout.unwrap_or(30),
        }
    }
}

impl Default for Builder<BuilderEmpty> {
    fn default() -> Self {
        Self::new()
    }
}

/// The final configuration object
pub struct Config {
    pub name: String,
    pub description: String,
    pub timeout: u64,
}

/// A token type used for capability-based access control
///
/// INVARIANT: Operations require specific capability tokens.
/// Tokens must be acquired in order and cannot be cloned.
#[must_use]
pub struct Token {
    level: u8,
}

impl Token {
    /// Create a level 1 token (entry level)
    pub fn level1() -> Self {
        println!("Creating level 1 token");
        Token { level: 1 }
    }

    /// Upgrade to level 2 (consumes level 1)
    ///
    /// ORDERING INVARIANT: Must have level 1 before getting level 2
    pub fn upgrade_to_level2(self) -> Token {
        assert_eq!(self.level, 1, "Must be level 1 to upgrade to level 2");
        println!("Upgrading to level 2");
        Token { level: 2 }
    }

    /// Upgrade to level 3 (consumes level 2)
    ///
    /// ORDERING INVARIANT: Must have level 2 before getting level 3
    pub fn upgrade_to_level3(self) -> Token {
        assert_eq!(self.level, 2, "Must be level 2 to upgrade to level 3");
        println!("Upgrading to level 3");
        Token { level: 3 }
    }

    /// Perform admin operation (requires level 3)
    ///
    /// INVARIANT: Admin operations require the highest level token
    pub fn admin_operation(self) {
        assert_eq!(self.level, 3, "Admin operation requires level 3 token");
        println!("Performing admin operation");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_typestate() {
        let config = Builder::new()
            .name("MyConfig")
            .description("A test configuration")
            .timeout(60)
            .complete()
            .build();

        assert_eq!(config.name, "MyConfig");
        assert_eq!(config.timeout, 60);
    }

    #[test]
    fn test_token_ordering() {
        let token = Token::level1();
        let token = token.upgrade_to_level2();
        let token = token.upgrade_to_level3();
        token.admin_operation();
    }
}
