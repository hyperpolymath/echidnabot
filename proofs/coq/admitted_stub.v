(* SPDX-License-Identifier: MPL-2.0 *)
(* Dogfood fixture: deliberately admitted Coq proof. coqc exits 0 with
   an admit warning; echidnabot's axiom scanner (src/trust/axiom_tracker)
   detects the Admitted. token and — once Regulator mode is wired — will
   block merges. Until then this round-trips through CI with a warning. *)

Theorem unproven : forall (P Q : Prop), P -> Q.
Proof.
  intros P Q HP.
Admitted.
