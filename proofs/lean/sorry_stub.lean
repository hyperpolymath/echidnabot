-- SPDX-License-Identifier: PMPL-1.0-or-later
-- Dogfood fixture: deliberately incomplete Lean4 proof using sorry.
-- Lean compiles with a warning; Regulator-mode axiom scan will treat
-- sorry as a blocking violation once wired.

theorem unproven (P Q : Prop) (_ : P) : Q := sorry
