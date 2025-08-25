use crate::data::incursion;
use crate::{config::Config, util::madness::Madness};
use std::sync::Arc;
use tokio::time::Duration;

pub struct IncursionUpdater {
    db: Arc<crate::DB>,
    config: Config,
}

impl IncursionUpdater {
    pub fn new(db: Arc<crate::DB>, config: Config) -> IncursionUpdater {
        IncursionUpdater { db, config }
    }

    pub fn start(self) {
        println!("Incursion updater starting with {} second interval", self.config.incursion_updater.interval_seconds);
        tokio::spawn(async move {
            // println!("Incursion updater background task spawned");
            self.run().await;
        });
    }

    async fn run(self) {
        // println!("Incursion updater main loop started");
        loop {
            // println!("Incursion updater: starting check cycle");
            let sleep_time = match self.run_once().await {
                Ok(()) => {
                    println!("Incursion updater: check completed successfully, sleeping for {} seconds", self.config.incursion_updater.interval_seconds);
                    self.config.incursion_updater.interval_seconds
                },
                Err(e) => {
                    // println!("Error in incursion updater: {:#?}", e);
                    // println!("Incursion updater: error occurred, sleeping for 3600 seconds before retry");
                    3600 // On error, wait 1 hour before retrying
                }
            };

            tokio::time::sleep(Duration::from_secs(sleep_time)).await;
        }
    }

    async fn run_once(&self) -> Result<(), Madness> {
        // println!("Starting incursion focus check...");
        
        // Check and update focus status
        match incursion::check_and_update_focus(&self.create_app()).await {
            Ok(()) => {
                // println!("Incursion focus check completed successfully");
                Ok(())
            }
            Err(e) => {
                // println!("Failed to check incursion focus: {:#?}", e);
                Err(e)
            }
        }
    }

    fn create_app(&self) -> crate::app::Application {
        crate::app::new(self.db.clone(), self.config.clone())
    }
}
