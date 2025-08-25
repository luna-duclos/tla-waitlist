use crate::data::srp;
use crate::{config::Config, util::madness::Madness};
use std::sync::Arc;
use tokio::time::Duration;

pub struct SRPUpdater {
    db: Arc<crate::DB>,
    config: Config,
}

impl SRPUpdater {
    pub fn new(db: Arc<crate::DB>, config: Config) -> SRPUpdater {
        SRPUpdater { db, config }
    }

    pub fn start(self) {
        tokio::spawn(async move {
            self.run().await;
        });
    }

    async fn run(self) {
        loop {
            let sleep_time = match self.run_once().await {
                Ok(()) => self.config.srp_updater.interval_seconds,
                Err(e) => {
                    error!("Error in SRP updater: {:#?}", e);
                    60 // On error, wait 1 minute before retrying
                }
            };

            tokio::time::sleep(Duration::from_secs(sleep_time)).await;
        }
    }

    fn get_db(&self) -> &crate::DB {
        &self.db
    }

    async fn run_once(&self) -> Result<(), Madness> {
        info!("Starting SRP payment processing...");
        
        // Check if we have a service account configured
        let service_account = srp::get_service_account_info(&self.create_app()).await?;
        if service_account.is_none() {
            info!("No SRP service account configured, skipping update");
            return Ok(());
        }

        // Process SRP payments
        srp::process_srp_payments(&self.create_app()).await?;
        
        // Check and update SRP validity based on focus status
        srp::check_srp_validity(&self.create_app()).await?;
        
        info!("SRP payment processing completed successfully");
        Ok(())
    }

    fn create_app(&self) -> crate::app::Application {
        crate::app::new(self.db.clone(), self.config.clone())
    }
}
