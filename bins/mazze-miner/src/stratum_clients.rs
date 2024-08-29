use futures::{SinkExt, StreamExt};
use log::{debug, error, info, trace, warn};
use mazze_types::{H256, U256};
use mazzecore::pow::{
    validate, PowComputer, ProofOfWorkProblem, ProofOfWorkSolution,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};
use tokio_util::codec::{Framed, LinesCodec};

pub struct StratumClient {
    framed: Framed<TcpStream, LinesCodec>,
    pow: Arc<PowComputer>,
    current_job: Option<ProofOfWorkProblem>,
    stratum_secret: String,
}

impl StratumClient {
    pub async fn connect(
        addr: &str, stratum_secret: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        info!("Attempting to connect to {}", addr);
        let stream = TcpStream::connect(addr).await?;
        info!("Connected successfully to {}", addr);
        let framed = Framed::new(stream, LinesCodec::new());
        let pow = Arc::new(PowComputer::new());
        Ok(StratumClient {
            framed,
            pow,
            current_job: None,
            stratum_secret: stratum_secret.to_string(),
        })
    }

    async fn subscribe(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Sending subscription request");
        let request = json!({
            "id": 1,
            "method": "mining.subscribe",
            "params": ["999", self.stratum_secret]
        });
        let request_json = serde_json::to_string(&request)?;
        trace!("Subscription request JSON: {}", request_json);
        self.framed.send(request_json).await?;
        info!("Subscription request sent");

        match self.receive_message().await? {
            Some(message) => {
                let value: Value = serde_json::from_str(&message)?;
                if let Some(result) = value.get("result") {
                    if result.as_bool() == Some(true) {
                        info!("Subscribed successfully");
                        Ok(())
                    } else {
                        warn!("Subscription failed");
                        Err("Subscription failed".into())
                    }
                } else {
                    error!("Invalid subscription response");
                    Err("Invalid subscription response".into())
                }
            }
            None => {
                error!("No response received for subscription");
                Err("No response received for subscription".into())
            }
        }
    }

    async fn handle_job_notification(
        &mut self, params: &[Value],
    ) -> Result<(), Box<dyn std::error::Error>> {
        match self.parse_job(params) {
            Ok(problem) => {
                self.current_job = Some(problem);
                if let Some(solution) = self.mine_job(&problem) {
                    self.submit_share(&solution).await?;
                }
                Ok(())
            }
            Err(e) => {
                error!("Failed to parse job: {}", e);
                Err(e.into())
            }
        }
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.subscribe().await?;

        loop {
            match self.receive_message().await? {
                Some(message) => {
                    debug!("Received message: {}", message);
                    let value: Value = serde_json::from_str(&message)?;

                    if let Some(method) =
                        value.get("method").and_then(Value::as_str)
                    {
                        match method {
                            "mining.notify" => {
                                if let Some(params) = value
                                    .get("params")
                                    .and_then(Value::as_array)
                                {
                                    self.handle_job_notification(params)
                                        .await?;
                                }
                            }
                            _ => debug!("Received unknown method: {}", method),
                        }
                    } else if let Some(result) = value.get("result") {
                        // This might be a response to our subscription or share submission
                        debug!("Received result: {:?}", result);
                    } else {
                        debug!("Received unknown message: {}", message);
                    }
                }
                None => {
                    info!("Server closed the connection");
                    break;
                }
            }
        }

        Ok(())
    }

    async fn receive_message(
        &mut self,
    ) -> Result<Option<String>, Box<dyn std::error::Error>> {
        match timeout(Duration::from_secs(30), self.framed.next()).await {
            Ok(Some(line_result)) => Ok(Some(line_result?)),
            Ok(None) => Ok(None),
            Err(_) => Err("Timeout waiting for message".into()),
        }
    }

    fn parse_job(
        &self, params: &[Value],
    ) -> Result<ProofOfWorkProblem, String> {
        if params.len() < 4 {
            return Err("Invalid job data: not enough parameters".into());
        }

        let pow_hash_str =
            params[2].as_str().ok_or("Invalid pow_hash: not a string")?;
        let boundary_str =
            params[3].as_str().ok_or("Invalid boundary: not a string")?;

        let pow_hash = H256::from_slice(
            &hex::decode(pow_hash_str.trim_start_matches("0x"))
                .map_err(|e| format!("Invalid pow_hash: {}", e))?,
        );

        let boundary_str = boundary_str.trim_start_matches("0x");
        let boundary_str = if boundary_str.len() % 2 != 0 {
            format!("0{}", boundary_str)
        } else {
            boundary_str.to_string()
        };

        let boundary_bytes = hex::decode(&boundary_str)
            .map_err(|e| format!("Invalid boundary hex: {}", e))?;
        let mut padded_bytes = [0u8; 32];
        padded_bytes[32 - boundary_bytes.len()..]
            .copy_from_slice(&boundary_bytes);
        let boundary = U256::from_big_endian(&padded_bytes);

        let block_height = params[1]
            .as_str()
            .ok_or("Invalid block height: not a string")?
            .parse::<u64>()
            .map_err(|e| format!("Invalid block height: {}", e))?;

        Ok(ProofOfWorkProblem::new(block_height, pow_hash, boundary))
    }

    fn mine_job(
        &self, problem: &ProofOfWorkProblem,
    ) -> Option<ProofOfWorkSolution> {
        info!(
            "Starting mining attempt with difficulty: {:?}",
            problem.difficulty
        );
        let start_time = std::time::Instant::now();
        let mut nonce = U256::zero();
        let mut hashes_checked = 0;

        while start_time.elapsed() < std::time::Duration::from_secs(10) {
            let hash = self.pow.compute(&nonce, &problem.block_hash);
            hashes_checked += 1;

            if ProofOfWorkProblem::validate_hash_against_boundary(
                &hash,
                &nonce,
                &problem.boundary,
            ) {
                info!(
                    "Solution found after checking {} hashes",
                    hashes_checked
                );
                return Some(ProofOfWorkSolution { nonce });
            }

            nonce = nonce.overflowing_add(U256::one()).0;
        }

        None
    }

    async fn submit_share(
        &mut self, solution: &ProofOfWorkSolution,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(problem) = &self.current_job {
            info!("Submitting share for job: {}", problem.block_height);
            let request = json!({
                "id": 4,
                "method": "mining.submit",
                "params": [
                    "worker1",
                    problem.block_height.to_string(),
                    format!("0x{:x}", solution.nonce),
                ]
            });
            let request_json = serde_json::to_string(&request)?;
            trace!("Submit share request JSON: {}", request_json);
            self.framed.send(request_json).await?;
            info!("Share submission sent");
        } else {
            warn!("Attempted to submit share without a current job");
        }
        Ok(())
    }
}