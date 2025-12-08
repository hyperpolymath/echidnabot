;;; STATE.scm - Conversation Checkpoint for echidnabot
;;; Format: Guile Scheme S-expressions
;;; License: MIT / Palimpsest-0.8

;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
;;; METADATA
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

(metadata
  (format-version . "1.0")
  (created . "2025-12-08")
  (last-updated . "2025-12-08")
  (generator . "claude-opus-4"))

;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
;;; PROJECT IDENTITY
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

(project
  (name . "echidnabot")
  (description . "UNDEFINED - awaiting user input")
  (repository . "hyperpolymath/echidnabot")
  (category . "bot/automation")  ; assumed from name
  (status . "ideation")
  (completion . 0))

;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
;;; CURRENT POSITION
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

(current-position
  (phase . "pre-development")
  (state . "empty-repository")
  (branch . "claude/create-state-scm-013QGU2z6VAUvPXb7VabscwU")

  (what-exists
    (git-repo . #t)
    (remote-configured . #t)
    (source-code . #f)
    (documentation . #f)
    (tests . #f)
    (ci-cd . #f)
    (dependencies . #f))

  (blockers
    ("Project scope and purpose undefined")
    ("Tech stack not selected")
    ("No requirements documented")))

;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
;;; ROUTE TO MVP v1
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

(mvp-v1
  (status . "not-started")
  (target-completion . "TBD")

  ;; MVP definition pending answers to questions below
  (core-features
    ("PLACEHOLDER: Define core bot functionality")
    ("PLACEHOLDER: Define integration targets")
    ("PLACEHOLDER: Define user interaction model"))

  (milestones
    (m0 (name . "Project Setup")
        (status . "in-progress")
        (tasks
          ("Create STATE.scm" . "in-progress")
          ("Define project scope" . "blocked")
          ("Select tech stack" . "blocked")
          ("Initialize project structure" . "pending")
          ("Set up development environment" . "pending")))

    (m1 (name . "Core Bot Infrastructure")
        (status . "pending")
        (tasks
          ("PLACEHOLDER: depends on project definition")))

    (m2 (name . "Primary Feature Set")
        (status . "pending")
        (tasks
          ("PLACEHOLDER: depends on project definition")))

    (m3 (name . "MVP Release")
        (status . "pending")
        (tasks
          ("Integration testing" . "pending")
          ("Documentation" . "pending")
          ("Initial deployment" . "pending")))))

;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
;;; KNOWN ISSUES
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

(issues
  (critical
    (issue-001
      (title . "Project purpose undefined")
      (description . "Cannot proceed without knowing what echidnabot should do")
      (blocking . ("all development work"))))

  (high
    (issue-002
      (title . "Tech stack not selected")
      (description . "Need to choose language, framework, and hosting approach")
      (options . ("Python + discord.py/telegram-bot"
                  "TypeScript + discord.js"
                  "Rust + serenity"
                  "Elixir + nostrum"
                  "Other - user specified"))))

  (medium
    (issue-003
      (title . "No CI/CD pipeline")
      (description . "Need to set up automated testing and deployment"))

    (issue-004
      (title . "No contribution guidelines")
      (description . "Need CONTRIBUTING.md if open source")))

  (low))

;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
;;; QUESTIONS FOR USER
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

(questions
  (priority-1-blocking
    (q1 (question . "What is echidnabot?")
        (context . "I need to understand the core purpose and functionality")
        (examples . ("Discord bot for moderation?"
                     "Telegram bot for notifications?"
                     "GitHub bot for automation?"
                     "IRC bot?"
                     "Multi-platform bot?"
                     "Something else entirely?")))

    (q2 (question . "What platform(s) should it target?")
        (context . "This determines dependencies and architecture")
        (options . ("Discord" "Telegram" "Slack" "IRC" "Matrix" "GitHub" "Multiple")))

    (q3 (question . "What are the 3-5 core features for MVP?")
        (context . "Need to scope the minimum viable product")
        (format . "List the essential functionality without which the bot is useless")))

  (priority-2-important
    (q4 (question . "What programming language/tech stack do you prefer?")
        (context . "Your preferences from state.scm suggest Rust, Elixir, or Haskell")
        (my-recommendation . "Rust with serenity for Discord, or Elixir for high concurrency"))

    (q5 (question . "Should this be self-hosted or cloud-deployed?")
        (options . ("Self-hosted (VPS/home server)"
                    "Cloud functions (AWS Lambda, Cloudflare Workers)"
                    "Container-based (Docker/Podman on K8s)"
                    "Managed bot hosting platform")))

    (q6 (question . "What's the expected scale?")
        (context . "Affects architecture decisions")
        (options . ("Personal/small server (<100 users)"
                    "Medium (100-10k users)"
                    "Large (10k+ users)"))))

  (priority-3-nice-to-know
    (q7 (question . "Any existing bots or projects to draw inspiration from?")
        (context . "Helps understand desired UX and feature set"))

    (q8 (question . "Is this open source? What license?")
        (context . "Affects documentation and contribution setup"))

    (q9 (question . "Any specific integrations needed?")
        (examples . ("Database" "External APIs" "Webhooks" "OAuth providers")))))

;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
;;; LONG-TERM ROADMAP
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

(roadmap
  (disclaimer . "Speculative until project scope is defined")

  (phase-0
    (name . "Foundation")
    (status . "current")
    (goals
      ("Define project scope and requirements")
      ("Select technology stack")
      ("Set up repository structure")
      ("Create development environment")
      ("Document architecture decisions")))

  (phase-1
    (name . "MVP Development")
    (status . "pending")
    (goals
      ("Implement core bot infrastructure")
      ("Build primary feature set")
      ("Basic error handling and logging")
      ("Initial test suite")
      ("Local deployment capability")))

  (phase-2
    (name . "Hardening")
    (status . "future")
    (goals
      ("Comprehensive test coverage")
      ("CI/CD pipeline")
      ("Production deployment")
      ("Monitoring and alerting")
      ("Performance optimization")))

  (phase-3
    (name . "Enhancement")
    (status . "future")
    (goals
      ("Extended feature set")
      ("Plugin/extension system")
      ("Admin dashboard/web UI")
      ("Multi-instance support")
      ("Advanced integrations")))

  (phase-4
    (name . "Maturity")
    (status . "future")
    (goals
      ("Community contributions")
      ("Comprehensive documentation")
      ("Stable API")
      ("Long-term maintenance plan"))))

;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
;;; SESSION TRACKING
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

(session
  (id . "013QGU2z6VAUvPXb7VabscwU")
  (started . "2025-12-08")
  (actions-taken
    ("Explored repository structure")
    ("Found repository is empty/new")
    ("Fetched STATE.scm format guidance")
    ("Created initial STATE.scm")))

;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
;;; FILES CREATED/MODIFIED THIS SESSION
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

(files
  (created
    ("STATE.scm" . "2025-12-08"))
  (modified))

;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
;;; NEXT ACTIONS
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

(critical-next
  (action-1 . "User to answer blocking questions (q1-q3)")
  (action-2 . "Define MVP feature set based on answers")
  (action-3 . "Select tech stack")
  (action-4 . "Initialize project with chosen stack")
  (action-5 . "Create README.md with project description"))

;;; END STATE.scm
;;; Download this file at end of session, upload at start of next conversation
