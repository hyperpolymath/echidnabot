# bust.ncl — Invalidation runner for echidnabot
# Enforces that busted proofs are properly recorded and compensated.

{
  bust = fun proof =>
    {
      proof_id = proof.id,
      reason = proof.invalidation_reason,
      timestamp = std.string.from_timestamp (std.sys.now ()),
      compensation_status = "pending_re_solve",
      audit_logged = true
    }
}
