//! Command-line interface for the Aurora build system

use std::path::PathBuf;

/// Build profile
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Profile {
    /// Debug build
    Debug,
    /// Release build
    Release,
    /// Custom profile
    Custom(String),
}

impl Profile {
    /// Get optimization level
    pub fn opt_level(&self) -> u8 {
        match self {
            Profile::Debug => 0,
            Profile::Release => 3,
            Profile::Custom(_) => 2,
        }
    }

    /// Check if debug info should be included
    pub fn debug_info(&self) -> bool {
        matches!(self, Profile::Debug | Profile::Custom(_))
    }
}

/// Build command arguments
#[derive(Debug, Clone)]
pub struct BuildArgs {
    /// Project path
    pub path: PathBuf,
    /// Build profile
    pub profile: Profile,
    /// Target triple (e.g., x86_64-unknown-linux-gnu)
    pub target: Option<String>,
    /// Number of parallel jobs
    pub jobs: usize,
    /// Verbose output
    pub verbose: bool,
}

impl Default for BuildArgs {
    fn default() -> Self {
        Self {
            path: PathBuf::from("."),
            profile: Profile::Debug,
            target: None,
            jobs: num_cpus::get(),
            verbose: false,
        }
    }
}

/// Test command arguments
#[derive(Debug, Clone)]
pub struct TestArgs {
    /// Project path
    pub path: PathBuf,
    /// Test name filter
    pub filter: Option<String>,
    /// Number of test threads
    pub test_threads: usize,
    /// Verbose output
    pub verbose: bool,
}

impl Default for TestArgs {
    fn default() -> Self {
        Self {
            path: PathBuf::from("."),
            filter: None,
            test_threads: num_cpus::get(),
            verbose: false,
        }
    }
}

/// Aurora CLI commands
#[derive(Debug)]
pub enum Command {
    /// Initialize new project
    Init {
        /// Project name
        name: String,
        /// Project path
        path: PathBuf,
    },
    /// Build project
    Build(BuildArgs),
    /// Run project
    Run {
        /// Build arguments
        build: BuildArgs,
        /// Program arguments
        args: Vec<String>,
    },
    /// Test project
    Test(TestArgs),
    /// Benchmark project
    Bench {
        /// Benchmark name filter
        filter: Option<String>,
    },
    /// Format source code
    Fmt {
        /// Check only, don't write
        check: bool,
    },
    /// Lint source code
    Lint {
        /// Fix automatically
        fix: bool,
    },
    /// Generate documentation
    Doc {
        /// Open in browser
        open: bool,
    },
    /// Cross-compile for target
    Cross {
        /// Target triple
        target: String,
        /// Build arguments
        build: BuildArgs,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_opt_levels() {
        assert_eq!(Profile::Debug.opt_level(), 0);
        assert_eq!(Profile::Release.opt_level(), 3);
        assert_eq!(Profile::Custom("test".to_string()).opt_level(), 2);
    }

    #[test]
    fn test_profile_debug_info() {
        assert!(Profile::Debug.debug_info());
        assert!(!Profile::Release.debug_info());
        assert!(Profile::Custom("test".to_string()).debug_info());
    }

    #[test]
    fn test_build_args_default() {
        let args = BuildArgs::default();
        assert_eq!(args.profile, Profile::Debug);
        assert!(args.jobs > 0);
        assert!(!args.verbose);
    }

    #[test]
    fn test_test_args_default() {
        let args = TestArgs::default();
        assert!(args.filter.is_none());
        assert!(args.test_threads > 0);
    }
}
