(* SPDX-License-Identifier: PMPL-1.0-or-later *)
(* Dogfood fixture: trivial passing Coq proof — exercises the
   proofs/** path trigger in .github/workflows/echidnabot.yml. *)

Theorem identity : forall (A : Prop), A -> A.
Proof.
  intros A HA.
  exact HA.
Qed.
