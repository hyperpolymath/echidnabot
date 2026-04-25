# intend.ncl — Goal declaration runner for echidnabot
# Validates that session intentions are aligned with contractile declarations.

let goal_schema = {
  name | String,
  wishes | Array String,
  bot-mode | String,
  success-criteria | String
} in

{
  validate = fun goal =>
    if (builtins.typeof goal) != "record"
    then { error = "Goal must be a record" }
    else if !("name" in goal)
    then { error = "Goal missing 'name' field" }
    else { valid = true, goal = goal }
}
