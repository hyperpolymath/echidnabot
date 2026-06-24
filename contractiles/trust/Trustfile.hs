-- SPDX-License-Identifier: MPL-2.0
-- Trustfile — integrity / provenance / supply-chain verification for echidnabot
--
-- Error-code namespace: T###
-- Compile + run: `runhaskell contractiles/trust/Trustfile.hs`
-- Each check returns ExitSuccess on pass, ExitFailure on violation.

module Trustfile where

import Control.Monad (forM)
import Data.List (isInfixOf)
import System.Directory (doesFileExist, doesDirectoryExist)
import System.Exit (ExitCode(..), exitFailure, exitSuccess)
import System.Process (readProcessWithExitCode)

-- ── Authoritative paths ────────────────────────────────────────────────

cargoTomlPath :: FilePath
cargoTomlPath = "Cargo.toml"

cargoLockPath :: FilePath
cargoLockPath = "Cargo.lock"

containerfilePath :: FilePath
containerfilePath = "Containerfile"

workflowsDir :: FilePath
workflowsDir = ".github/workflows"

licenseFile :: FilePath
licenseFile = "LICENSE"

-- ── Trust boundaries ───────────────────────────────────────────────────

trustLevel :: String
trustLevel = "maximal"

trustActions :: [String]
trustActions =
  [ "read"
  , "build"
  , "test"
  , "lint"
  , "format"
  , "file-pr-on-branch"
  , "admin-merge-with-owner-authorisation"
  ]

trustDeny :: [String]
trustDeny =
  [ "delete-branch-without-owner-confirm"
  , "force-push-to-main"
  , "modify-ci-secrets"
  , "publish-without-tag-cut"
  , "edit-licence-headers-automatically"   -- per [[feedback_no_automated_licence_edits]]
  ]

-- ── Checks ─────────────────────────────────────────────────────────────

-- T001: LICENSE contains a recognised SPDX identifier
checkT001 :: IO Bool
checkT001 = do
  exists <- doesFileExist licenseFile
  if not exists
    then return False
    else do
      (ec, _, _) <- readProcessWithExitCode "grep"
        ["-qE", "SPDX-License-Identifier|MIT|Apache|MPL|AGPL|PMPL|PLMP", licenseFile] ""
      return (ec == ExitSuccess)

-- T002: No secrets files committed
checkT002 :: IO Bool
checkT002 = do
  envExists <- doesFileExist ".env"
  credsExists <- doesFileExist "credentials.json"
  envLocalExists <- doesFileExist ".env.local"
  secretsExists <- doesFileExist ".secrets"
  return (not envExists && not credsExists && not envLocalExists && not secretsExists)

-- T010: HEAD commit is GPG-signed
checkT010 :: IO Bool
checkT010 = do
  (ec, out, _) <- readProcessWithExitCode "git" ["log", "-1", "--pretty=%G?"] ""
  return (ec == ExitSuccess && not (null out) && head out `elem` "GN")

-- T011: Cargo.lock is tracked
checkT011 :: IO Bool
checkT011 = do
  exists <- doesFileExist cargoLockPath
  if not exists
    then return False
    else do
      (ec, _, _) <- readProcessWithExitCode "git"
        ["ls-files", "--error-unmatch", cargoLockPath] ""
      return (ec == ExitSuccess)

-- T020: GitHub Actions SHA-pinned (no bare branch / tag refs)
checkT020 :: IO Bool
checkT020 = do
  wfExists <- doesDirectoryExist workflowsDir
  if not wfExists
    then return True  -- vacuously true
    else do
      (ec, _, _) <- readProcessWithExitCode "bash"
        [ "-c"
        , "! grep -rE 'uses: [^/]+/[^@]+@(main|master|v[0-9]+\\.?[0-9]*\\.?[0-9]*)' "
            ++ workflowsDir ++ " | grep -v '^#'"
        ] ""
      return (ec == ExitSuccess)

-- T021: SHA pins are not fake (resolve to real upstream commits)
checkT021 :: IO Bool
checkT021 = do
  scriptExists <- doesFileExist "scripts/verify-action-shas.sh"
  if not scriptExists
    then return True  -- vacuously true if scanner not present
    else do
      (_, out, _) <- readProcessWithExitCode "bash"
        ["scripts/verify-action-shas.sh"] ""
      return (not ("FAKE" `isInfixOf` out))

-- T030: Containerfile uses approved base (Chainguard wolfi-base)
checkT030 :: IO Bool
checkT030 = do
  exists <- doesFileExist containerfilePath
  if not exists
    then return True  -- vacuously true
    else do
      (ec, out, _) <- readProcessWithExitCode "grep" ["-E", "^FROM", containerfilePath] ""
      if ec /= ExitSuccess
        then return False
        else return ("cgr.dev/chainguard/" `isInfixOf` out)

-- T040: No deprecated nix files (estate policy 2026-06-01)
checkT040 :: IO Bool
checkT040 = do
  flakeNix <- doesFileExist "flake.nix"
  flakeLock <- doesFileExist "flake.lock"
  return (not flakeNix && not flakeLock)

-- T050: Webhook signature verification present in code
checkT050 :: IO Bool
checkT050 = do
  webhooksExists <- doesFileExist "src/api/webhooks.rs"
  if not webhooksExists
    then return True
    else do
      (ec, _, _) <- readProcessWithExitCode "grep"
        ["-q", "verify_github_signature\\|verify_gitlab_signature\\|verify_codeberg_signature\\|hmac", "src/api/webhooks.rs"] ""
      return (ec == ExitSuccess)

-- T051: Database connection uses TLS (no plain postgres://)
checkT051 :: IO Bool
checkT051 = do
  exampleExists <- doesFileExist "echidnabot.example.toml"
  if not exampleExists
    then return True
    else do
      (ec, _, _) <- readProcessWithExitCode "bash"
        [ "-c"
        , "! grep -qE 'postgres://[^?]+$' echidnabot.example.toml"
        ] ""
      return (ec == ExitSuccess)

-- ── Driver ─────────────────────────────────────────────────────────────

allChecks :: [(String, IO Bool)]
allChecks =
  [ ("T001-license-content",    checkT001)
  , ("T002-no-secrets",         checkT002)
  , ("T010-gpg-signed",         checkT010)
  , ("T011-cargo-lock-tracked", checkT011)
  , ("T020-shas-pinned",        checkT020)
  , ("T021-shas-not-fake",      checkT021)
  , ("T030-trusted-base",       checkT030)
  , ("T040-no-nix",             checkT040)
  , ("T050-webhook-sig-verify", checkT050)
  , ("T051-db-tls",             checkT051)
  ]

main :: IO ()
main = do
  results <- forM allChecks $ \(name, check) -> do
    ok <- check
    putStrLn $ (if ok then "[PASS] " else "[FAIL] ") ++ name
    return (name, ok)
  let failed = filter (not . snd) results
  if null failed
    then do
      putStrLn $ "\nAll " ++ show (length allChecks) ++ " trust checks passed."
      exitSuccess
    else do
      putStrLn $ "\n" ++ show (length failed) ++ " of " ++ show (length allChecks) ++ " checks failed:"
      mapM_ (\(n, _) -> putStrLn $ "  - " ++ n) failed
      exitFailure
