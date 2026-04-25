# adjust.ncl — Modification protocol runner for echidnabot
# Validates proof adjustments maintain correctness and traceability.

let adjustment_schema = {
  proof_id | String,
  trigger | String,
  old_confidence | Number,
  new_confidence | Number,
  rationale | String
} in

{
  can_adjust = fun adjustment =>
    if adjustment.new_confidence > adjustment.old_confidence
    then true
    else if std.string.contains adjustment.trigger "simplification"
    then true
    else false,
  
  verify_adjustment = fun adjustment =>
    if !(std.record.has_field "rationale" adjustment)
    then { error = "Rationale required for adjustment" }
    else { valid = true }
}
