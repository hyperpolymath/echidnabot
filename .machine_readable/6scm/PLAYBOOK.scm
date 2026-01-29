;; SPDX-License-Identifier: PMPL-1.0-or-later
;; PLAYBOOK.scm - Operational runbook for echidnabot

(define playbook
  `((version . "1.0.0")
    (updated . "2026-01-29")

    (quick-reference
      ((emergency-contact . "jonathan.jewell@open.ac.uk")
       (status-page . "https://status.echidna.dev")
       (monitoring . "https://grafana.echidna.dev")
       (logs . "https://logs.echidna.dev")))

    (deployment-procedures
      ((development-setup
         ((prerequisites
            . ("Rust 1.75+"
              "PostgreSQL 14+ or Docker"
              "ECHIDNA instance (or run locally)"))
          (steps
            . ("1. Clone repository: git clone https://github.com/hyperpolymath/echidnabot"
              "2. Set up database: createdb echidnabot && sqlx migrate run"
              "3. Configure env: cp .env.example .env && edit DATABASE_URL, ECHIDNA_URL"
              "4. Build: cargo build"
              "5. Run tests: cargo test"
              "6. Start server: cargo run -- serve --port 3000"
              "7. Test webhook: curl -X POST http://localhost:3000/health"))))

       (docker-compose-setup
         ((description . "One-command local environment")
          (file . "docker-compose.yml")
          (steps
            . ("1. Copy docker-compose.yml to project root"
              "2. Run: docker-compose up -d"
              "3. Services: postgres:5432, echidna:8080, echidnabot:3000"
              "4. Check health: curl http://localhost:3000/health"
              "5. View logs: docker-compose logs -f echidnabot"
              "6. Stop: docker-compose down"))))

       (production-deployment
         ((platform . "Kubernetes or Docker Swarm")
          (prerequisites
            . ("PostgreSQL cluster with backups"
              "ECHIDNA production instance"
              "TLS certificates for webhook endpoint"
              "Secrets configured (GitHub App key, GitLab token, etc.)"))
          (steps
            . ("1. Build production image: docker build -t echidnabot:latest ."
              "2. Push to registry: docker push registry.example.com/echidnabot:latest"
              "3. Update secrets: kubectl create secret generic echidnabot-secrets --from-env-file=.env.prod"
              "4. Deploy: kubectl apply -f k8s/echidnabot.yaml"
              "5. Check pods: kubectl get pods -l app=echidnabot"
              "6. Check logs: kubectl logs -f deployment/echidnabot"
              "7. Test webhook: curl -X POST https://webhooks.example.com/health"))
          (health-checks
            . ("GET /health - Basic health (200 OK)"
              "GET /health/ready - Database + ECHIDNA connectivity"
              "GET /metrics - Prometheus metrics"))
          (rollout-strategy . "Blue-green with 5-minute soak time")
          (rollback-procedure . "kubectl rollout undo deployment/echidnabot"))))

      (database-management
        ((migrations
           ((create . "sqlx migrate add <name>")
            (apply . "sqlx migrate run")
            (revert . "sqlx migrate revert")
            (status . "sqlx migrate info")))

         (backup
           ((frequency . "Daily at 2 AM UTC")
            (retention . "30 days")
            (command . "pg_dump echidnabot | gzip > backup-$(date +%Y%m%d).sql.gz")
            (restore . "gunzip < backup-20260129.sql.gz | psql echidnabot")
            (verification . "pg_restore --list backup.sql.gz")
            (offsite . "Copy to S3/GCS/Azure after each backup")))

         (monitoring
           ((queries . "SELECT COUNT(*) FROM jobs WHERE status = 'pending'")
            (alerts . ("Job queue length > 1000"
                      "Pending jobs older than 10 minutes"
                      "Database connection failures"))))))

      (webhook-configuration
        ((github-app-setup
           ((steps
             . ("1. Go to GitHub Settings > Developer > GitHub Apps > New"
               "2. Name: echidnabot"
               "3. Homepage URL: https://github.com/hyperpolymath/echidnabot"
               "4. Webhook URL: https://webhooks.example.com/webhook"
               "5. Webhook secret: Generate strong random secret"
               "6. Permissions: Checks (read/write), Contents (read), Pull requests (read/write)"
               "7. Events: Push, Pull request, Check run"
               "8. Download private key"
               "9. Install app on repositories"
               "10. Set env vars: GITHUB_APP_ID, GITHUB_PRIVATE_KEY_PATH, GITHUB_WEBHOOK_SECRET"))))

         (gitlab-webhook-setup
           ((steps
             . ("1. Go to GitLab Project > Settings > Webhooks"
               "2. URL: https://webhooks.example.com/webhook/gitlab"
               "3. Secret token: Generate strong random secret"
               "4. Triggers: Push events, Merge request events"
               "5. Enable SSL verification"
               "6. Set env var: GITLAB_TOKEN (project access token with api scope)"))))

         (bitbucket-webhook-setup
           ((steps
             . ("1. Go to Bitbucket Repository > Settings > Webhooks"
               "2. URL: https://webhooks.example.com/webhook/bitbucket"
               "3. Triggers: Repository push, Pull request created/updated"
               "4. Set env var: BITBUCKET_TOKEN (app password with webhook, pullrequest permissions)"))))))

      (echidna-integration
        ((configuration
           ((env-var . "ECHIDNA_URL=http://echidna:8080")
            (timeout . "300 seconds")
            (retry-strategy . "3 attempts with exponential backoff")
            (health-check . "GET /api/health every 30 seconds")))

         (troubleshooting
           ((echidna-unavailable
              . ("Check ECHIDNA_URL is correct"
                "Verify ECHIDNA service is running: curl http://echidna:8080/api/health"
                "Check network connectivity between echidnabot and ECHIDNA"
                "Review ECHIDNA logs for errors"
                "Restart ECHIDNA if needed: docker restart echidna"))
            (verification-timeout
              . ("Check prover backend is responsive"
                "Increase timeout if theorem is complex"
                "Check container resource limits"
                "Review ECHIDNA prover logs"))
            (invalid-response
              . ("Verify ECHIDNA API version compatibility"
                "Check response format matches expected schema"
                "Review network logs for corruption")))))))

    (monitoring-and-alerting
      ((key-metrics
         ((webhook-throughput . "webhooks_received_total (counter)")
          (verification-duration . "verification_duration_seconds (histogram)")
          (verification-success-rate . "verification_success_total / verification_total")
          (job-queue-length . "job_queue_length (gauge)")
          (echidna-api-latency . "echidna_api_call_duration_seconds (histogram)")
          (database-connection-pool . "db_connections_active (gauge)")))

       (alerts
         ((critical
            . ("Job queue length > 1000 for 5 minutes"
              "Webhook endpoint down (health check fails)"
              "Database connection failures > 10/minute"
              "ECHIDNA API unavailable for 1 minute"
              "Verification failure rate > 50% for 10 minutes"))
          (warning
            . ("Job queue length > 500 for 5 minutes"
              "Pending jobs older than 5 minutes"
              "ECHIDNA API latency > 5 seconds (p95)"
              "Database connection pool exhausted"))
          (info
            . ("New repository registered"
              "Webhook signature verification failed (potential attack)"
              "Unusual verification pattern detected"))))

       (dashboards
         ((grafana-echidnabot
            . ("Webhook throughput (requests/second)"
              "Verification latency (p50, p95, p99)"
              "Job queue length over time"
              "Success/failure rate by prover"
              "ECHIDNA API call latency"
              "Database query performance"))))))

    (incident-response
      ((high-webhook-volume
         ((symptoms . "Job queue growing, webhooks queuing, latency increasing")
          (diagnosis . "Check webhook source, look for spam or attack")
          (mitigation
            . ("Enable rate limiting per repository"
              "Temporarily disable webhooks from spammy repos"
              "Scale up echidnabot instances"
              "Increase job queue workers"))
          (prevention . "Implement rate limiting by default (100/min per repo)")))

       (echidna-unavailable
         ((symptoms . "All verifications failing, ECHIDNA health check fails")
          (diagnosis . "ECHIDNA service down or unreachable")
          (mitigation
            . ("Check ECHIDNA service: systemctl status echidna"
              "Review ECHIDNA logs: journalctl -u echidna -f"
              "Restart ECHIDNA if needed: systemctl restart echidna"
              "If persistent, rollback ECHIDNA to previous version"
              "echidnabot will retry failed jobs when ECHIDNA recovers"))
          (prevention . "Deploy ECHIDNA with redundancy, health monitoring")))

       (database-connection-exhaustion
         ((symptoms . "500 errors from API, 'too many connections' in logs")
          (diagnosis . "Connection pool exhausted, possible leak or high load")
          (mitigation
            . ("Increase PostgreSQL max_connections"
              "Increase echidnabot connection pool size"
              "Restart echidnabot to clear potential leaks"
              "Review long-running transactions"))
          (prevention . "Connection pool limits, transaction timeouts, monitoring")))

       (webhook-signature-attacks
         ((symptoms . "Many signature verification failures in logs")
          (diagnosis . "Someone trying to forge webhooks or using wrong secret")
          (mitigation
            . ("Verify webhook secrets are correctly configured"
              "Check if platform (GitHub/GitLab) changed webhook format"
              "Review IP addresses of failed attempts"
              "Rate-limit or block suspicious IPs"))
          (prevention . "HMAC verification enforced, security monitoring")))

       (container-resource-exhaustion
         ((symptoms . "Verifications timing out, host running out of memory/CPU")
          (diagnosis . "Too many concurrent container verifications")
          (mitigation
            . ("Reduce concurrent job limit in config"
              "Kill runaway containers: docker ps | grep echidna-verify | xargs docker kill"
              "Increase host resources or distribute load"
              "Review verification timeouts"))
          (prevention . "Strict resource limits, concurrency limits, monitoring")))))

    (maintenance-procedures
      ((log-rotation
         ((frequency . "Daily")
          (retention . "30 days")
          (compression . "gzip after 1 day")
          (command . "logrotate /etc/logrotate.d/echidnabot")))

       (dependency-updates
         ((frequency . "Weekly check, monthly apply")
          (security-patches . "Apply immediately")
          (procedure
            . ("1. Check for updates: cargo outdated"
              "2. Review changelogs for breaking changes"
              "3. Update Cargo.toml versions"
              "4. Run tests: cargo test"
              "5. Update Cargo.lock: cargo update"
              "6. Deploy to staging first"
              "7. Monitor for 24 hours"
              "8. Deploy to production"))))

       (database-vacuum
         ((frequency . "Weekly")
          (command . "VACUUM ANALYZE")
          (full-vacuum . "Monthly, during low-traffic window")
          (monitoring . "Check table bloat: SELECT * FROM pg_stat_user_tables")))

       (pruning-old-jobs
         ((frequency . "Weekly")
          (retention . "Completed jobs older than 90 days")
          (command . "DELETE FROM jobs WHERE status = 'completed' AND updated_at < NOW() - INTERVAL '90 days'")))

       (certificate-renewal
         ((webhook-endpoint . "Let's Encrypt auto-renewal via certbot")
          (monitoring . "Alert 30 days before expiry")
          (procedure . "certbot renew --dry-run (test), certbot renew (apply)")))))

    (debugging-procedures
      ((webhook-not-received
         ((checklist
            . ("1. Verify webhook is configured in GitHub/GitLab/Bitbucket"
              "2. Check webhook URL is publicly accessible"
              "3. Review platform webhook delivery logs"
              "4. Check echidnabot logs for incoming requests"
              "5. Verify signature verification not rejecting webhooks"
              "6. Test manually: curl -X POST with sample payload"))))

       (verification-failing
         ((checklist
            . ("1. Check ECHIDNA is reachable: curl http://echidna:8080/api/health"
              "2. Review proof file syntax (is it valid Coq/Lean/etc?)"
              "3. Check prover backend is configured correctly in ECHIDNA"
              "4. Review ECHIDNA logs for detailed error"
              "5. Try manual verification: echidna verify --prover coq --file proof.v"
              "6. Check container has correct prover installed"))))

       (job-stuck-pending
         ((checklist
            . ("1. Check job_queue_length metric (is worker running?)"
              "2. Review database: SELECT * FROM jobs WHERE status = 'pending' ORDER BY created_at"
              "3. Check worker logs for errors"
              "4. Verify database connectivity"
              "5. Restart worker if needed"))))

       (graphql-api-errors
         ((checklist
            . ("1. Check query syntax in GraphQL Playground"
              "2. Review API logs for detailed error"
              "3. Verify authentication if required"
              "4. Check database connection"
              "5. Test with minimal query to isolate issue")))))))

    (security-procedures
      ((secret-rotation
         ((frequency . "Quarterly or on suspected compromise")
          (procedure
            . ("1. Generate new secret (webhook or API token)"
              "2. Update in echidnabot configuration"
              "3. Update in GitHub/GitLab/Bitbucket webhook settings"
              "4. Monitor for signature failures"
              "5. Verify webhooks working with new secret"
              "6. Document rotation in changelog"))))

       (access-control-review
         ((frequency . "Quarterly")
          (procedure
            . ("1. Review GitHub App installation list"
              "2. Revoke access from inactive repositories"
              "3. Review database user permissions"
              "4. Audit admin access to production systems"
              "5. Review secret access (who has env vars, keys)"))))

       (security-audit
         ((frequency . "Annual or after security incidents")
          (scope
            . ("Dependency vulnerabilities: cargo audit"
              "Code vulnerabilities: cargo clippy --  -W clippy::all"
              "Container security: docker scan echidnabot:latest"
              "Webhook signature verification correctness"
              "SQL injection vulnerability assessment"
              "Container escape vulnerability testing"))))))))
