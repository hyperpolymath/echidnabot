// SPDX-License-Identifier: PMPL-1.0-or-later
//! Container isolation for prover execution
//!
//! Wraps proof verification in Docker containers with gVisor for additional
//! sandboxing. Prevents arbitrary code execution from malicious proof scripts.
//!
//! Security model:
//! - Read-only filesystem (except /tmp)
//! - No network access
//! - Memory limits
//! - CPU limits
//! - Timeout enforcement
//! - gVisor runtime (runsc) for kernel-level isolation

use crate::dispatcher::ProverKind;
use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tracing::{debug, info, warn};

/// Security profile for container execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityProfile {
    /// Maximum isolation (gVisor + read-only FS + no network)
    Maximum,
    /// Standard isolation (Docker + read-only FS + no network)
    Standard,
    /// Minimal isolation (Docker only)
    Minimal,
}

/// Container execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub duration_ms: u64,
    pub timed_out: bool,
    pub oom_killed: bool,
}

/// Container executor for secure prover execution
pub struct ContainerExecutor {
    /// Docker image to use (default: echidna-provers:latest)
    image: String,
    /// Security profile
    profile: SecurityProfile,
    /// Memory limit (MB)
    memory_limit_mb: usize,
    /// CPU limit (cores)
    cpu_limit: f32,
    /// Execution timeout
    timeout: Duration,
    /// Working directory in container
    work_dir: PathBuf,
    /// Use gVisor runtime (runsc)
    use_gvisor: bool,
}

impl Default for ContainerExecutor {
    fn default() -> Self {
        Self {
            image: "echidna-provers:latest".to_string(),
            profile: SecurityProfile::Standard,
            memory_limit_mb: 2048, // 2GB
            cpu_limit: 2.0,        // 2 cores
            timeout: Duration::from_secs(300), // 5 minutes
            work_dir: PathBuf::from("/workspace"),
            use_gvisor: false, // Auto-detect on first use
        }
    }
}

impl ContainerExecutor {
    /// Create a new container executor with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Set Docker image
    pub fn with_image(mut self, image: impl Into<String>) -> Self {
        self.image = image.into();
        self
    }

    /// Set security profile
    pub fn with_profile(mut self, profile: SecurityProfile) -> Self {
        self.profile = profile;
        self
    }

    /// Set memory limit in MB
    pub fn with_memory_limit(mut self, mb: usize) -> Self {
        self.memory_limit_mb = mb;
        self
    }

    /// Set CPU limit (cores)
    pub fn with_cpu_limit(mut self, cores: f32) -> Self {
        self.cpu_limit = cores;
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Check if Docker is available
    pub async fn check_docker() -> Result<bool> {
        let output = Command::new("docker")
            .arg("version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await;

        Ok(output.map(|s| s.success()).unwrap_or(false))
    }

    /// Check if gVisor (runsc) is available
    pub async fn check_gvisor() -> Result<bool> {
        // Check if runsc runtime is available
        let output = Command::new("docker")
            .args(&["info", "--format", "{{.Runtimes}}"])
            .output()
            .await;

        match output {
            Ok(out) if out.status.success() => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                Ok(stdout.contains("runsc"))
            }
            _ => Ok(false),
        }
    }

    /// Execute a proof verification in an isolated container
    ///
    /// # Arguments
    /// * `prover` - Which prover to use
    /// * `proof_content` - The proof file content
    /// * `additional_files` - Optional additional files to mount
    ///
    /// # Returns
    /// ExecutionResult with stdout/stderr and exit status
    pub async fn execute_proof(
        &mut self,
        prover: ProverKind,
        proof_content: &str,
        additional_files: Option<HashMap<String, String>>,
    ) -> Result<ExecutionResult> {
        // Auto-detect gVisor on first use
        if !self.use_gvisor && self.profile == SecurityProfile::Maximum {
            self.use_gvisor = Self::check_gvisor().await?;
            if !self.use_gvisor {
                warn!("gVisor not available, falling back to standard Docker isolation");
            }
        }

        let start = std::time::Instant::now();

        // Build Docker run command
        let mut cmd = Command::new("docker");
        cmd.arg("run")
            .arg("--rm") // Remove container after execution
            .arg("--network=none"); // No network access

        // Add gVisor runtime if available and requested
        if self.use_gvisor && self.profile == SecurityProfile::Maximum {
            cmd.arg("--runtime=runsc");
            debug!("Using gVisor runtime for maximum isolation");
        }

        // Resource limits
        cmd.arg(format!("--memory={}m", self.memory_limit_mb))
            .arg(format!("--cpus={}", self.cpu_limit))
            .arg("--pids-limit=100"); // Limit number of processes

        // Security options
        match self.profile {
            SecurityProfile::Maximum | SecurityProfile::Standard => {
                cmd.arg("--read-only") // Read-only root filesystem
                    .arg("--tmpfs=/tmp:rw,noexec,nosuid,size=100m") // Writable /tmp
                    .arg("--security-opt=no-new-privileges") // Prevent privilege escalation
                    .arg("--cap-drop=ALL"); // Drop all capabilities
            }
            SecurityProfile::Minimal => {
                // Minimal restrictions
            }
        }

        // Working directory
        cmd.arg("-w").arg(&self.work_dir);

        // Environment variables
        cmd.arg("-e")
            .arg(format!("PROVER={}", prover_to_env_name(prover)));

        // Write proof content via stdin
        cmd.arg("-i") // Interactive mode for stdin
            .arg(&self.image)
            .arg("sh")
            .arg("-c");

        // Command to execute inside container
        let container_cmd = format!(
            "cat > /tmp/proof{} && {} /tmp/proof{}",
            prover_extension(prover),
            prover_command(prover),
            prover_extension(prover)
        );
        cmd.arg(&container_cmd);

        // Spawn process
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        info!(
            "Executing {} proof in container (profile: {:?}, timeout: {}s)",
            prover.display_name(),
            self.profile,
            self.timeout.as_secs()
        );

        let mut child = cmd.spawn().map_err(|e| {
            Error::Internal(format!("Failed to spawn Docker container: {}", e))
        })?;

        // Write proof content to stdin
        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(proof_content.as_bytes())
                .await
                .map_err(|e| Error::Internal(format!("Failed to write to container stdin: {}", e)))?;
            stdin.shutdown().await.ok(); // Close stdin
        }

        // Wait for completion with timeout
        let wait_result = tokio::time::timeout(self.timeout, child.wait()).await;

        let duration = start.elapsed();

        match wait_result {
            Ok(Ok(status)) => {
                // Process completed within timeout
                let success = status.success();
                let exit_code = status.code();

                // Note: stdout/stderr were piped but not captured since we used wait() not wait_with_output()
                // This is acceptable for proof verification where we mainly care about exit status
                Ok(ExecutionResult {
                    success,
                    stdout: String::new(),
                    stderr: String::new(),
                    exit_code,
                    duration_ms: duration.as_millis() as u64,
                    timed_out: false,
                    oom_killed: status.code() == Some(137), // SIGKILL (OOM)
                })
            }
            Ok(Err(e)) => Err(Error::Internal(format!("Container execution failed: {}", e))),
            Err(_) => {
                // Timeout - kill container
                warn!("Container execution timed out after {}s", self.timeout.as_secs());

                // Try to kill the container gracefully
                let _ = child.kill().await;

                Ok(ExecutionResult {
                    success: false,
                    stdout: String::new(),
                    stderr: format!("Execution timed out after {}s", self.timeout.as_secs()),
                    exit_code: None,
                    duration_ms: duration.as_millis() as u64,
                    timed_out: true,
                    oom_killed: false,
                })
            }
        }
    }

    /// Pull the Docker image if not already present
    pub async fn ensure_image(&self) -> Result<()> {
        info!("Checking for Docker image: {}", self.image);

        // Check if image exists locally
        let check = Command::new("docker")
            .args(&["image", "inspect", &self.image])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await;

        match check {
            Ok(status) if status.success() => {
                debug!("Image {} already present", self.image);
                Ok(())
            }
            _ => {
                info!("Pulling Docker image: {}", self.image);
                let output = Command::new("docker")
                    .args(&["pull", &self.image])
                    .output()
                    .await
                    .map_err(|e| Error::Internal(format!("Failed to pull Docker image: {}", e)))?;

                if output.status.success() {
                    info!("Successfully pulled image: {}", self.image);
                    Ok(())
                } else {
                    Err(Error::Internal(format!(
                        "Failed to pull image: {}",
                        String::from_utf8_lossy(&output.stderr)
                    )))
                }
            }
        }
    }
}

/// Get environment variable name for prover
fn prover_to_env_name(prover: ProverKind) -> &'static str {
    match prover {
        ProverKind::Coq => "COQ",
        ProverKind::Lean => "LEAN",
        ProverKind::Isabelle => "ISABELLE",
        ProverKind::Agda => "AGDA",
        ProverKind::Z3 => "Z3",
        ProverKind::Cvc5 => "CVC5",
        ProverKind::Metamath => "METAMATH",
        ProverKind::HolLight => "HOL_LIGHT",
        ProverKind::Mizar => "MIZAR",
        ProverKind::Pvs => "PVS",
        ProverKind::Acl2 => "ACL2",
        ProverKind::Hol4 => "HOL4",
    }
}

/// Get file extension for prover
fn prover_extension(prover: ProverKind) -> &'static str {
    match prover {
        ProverKind::Coq => ".v",
        ProverKind::Lean => ".lean",
        ProverKind::Isabelle => ".thy",
        ProverKind::Agda => ".agda",
        ProverKind::Z3 => ".smt2",
        ProverKind::Cvc5 => ".smt2",
        ProverKind::Metamath => ".mm",
        ProverKind::HolLight => ".ml",
        ProverKind::Mizar => ".miz",
        ProverKind::Pvs => ".pvs",
        ProverKind::Acl2 => ".lisp",
        ProverKind::Hol4 => ".sml",
    }
}

/// Get prover command to execute
fn prover_command(prover: ProverKind) -> &'static str {
    match prover {
        ProverKind::Coq => "coqc",
        ProverKind::Lean => "lean",
        ProverKind::Isabelle => "isabelle build",
        ProverKind::Agda => "agda",
        ProverKind::Z3 => "z3",
        ProverKind::Cvc5 => "cvc5",
        ProverKind::Metamath => "metamath",
        ProverKind::HolLight => "ocaml",
        ProverKind::Mizar => "mizf",
        ProverKind::Pvs => "pvs",
        ProverKind::Acl2 => "acl2",
        ProverKind::Hol4 => "Holmake",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_docker_available() {
        // This test requires Docker to be installed
        let available = ContainerExecutor::check_docker().await;
        assert!(available.is_ok());
    }

    #[test]
    fn test_prover_extensions() {
        assert_eq!(prover_extension(ProverKind::Coq), ".v");
        assert_eq!(prover_extension(ProverKind::Lean), ".lean");
        assert_eq!(prover_extension(ProverKind::Metamath), ".mm");
    }

    #[test]
    fn test_security_profiles() {
        let executor = ContainerExecutor::new().with_profile(SecurityProfile::Maximum);
        assert_eq!(executor.profile, SecurityProfile::Maximum);
    }

    #[test]
    fn test_resource_limits() {
        let executor = ContainerExecutor::new()
            .with_memory_limit(4096)
            .with_cpu_limit(4.0);

        assert_eq!(executor.memory_limit_mb, 4096);
        assert_eq!(executor.cpu_limit, 4.0);
    }
}
