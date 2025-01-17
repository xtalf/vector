use anyhow::Result;
use clap::Args;

use crate::testing::integration::{self, IntegrationTest, OldIntegrationTest};
use crate::testing::state::EnvsDir;

/// Stop an environment
#[derive(Args, Debug)]
#[command()]
pub struct Cli {
    /// The desired integration
    integration: String,

    /// The desired environment. If not present, all running environments are stopped.
    environment: Option<String>,

    /// Use the currently defined configuration if the environment is not up
    #[arg(short, long)]
    force: bool,
}

impl Cli {
    pub fn exec(self) -> Result<()> {
        // Temporary hack to run old-style integration tests
        if self.environment.is_none() && integration::old_exists(&self.integration)? {
            let integration = OldIntegrationTest::new(&self.integration);
            return integration.stop();
        }

        if let Some(environment) = self.environment {
            IntegrationTest::new(self.integration, environment)?.stop(self.force)
        } else {
            let envs = EnvsDir::new(&self.integration).list_active()?;
            if envs.is_empty() {
                println!("No environments for {:?} are active.", self.integration);
            } else {
                for environment in envs {
                    IntegrationTest::new(self.integration.clone(), environment)?
                        .stop(self.force)?;
                }
            }
            Ok(())
        }
    }
}
