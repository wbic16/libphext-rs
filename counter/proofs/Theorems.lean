import CounterKernel
import H5iAppLib
open Aeneas Aeneas.Std Result Aeneas.Std.WP H5iAppLib

-- `step*` specs for every `==` and `clone` the kernel derives.
h5i_derive_all

namespace counter_kernel

/-- The invariant: the counter never passes its limit. -/
def Inv (s : State) : Prop := s.count.val ≤ s.limit.val

/-- No command panics or overflows, and every one keeps the invariant. -/
theorem inv_preserved (s : State) (c : Command) (h : Inv s) :
    transition s c ⦃ r => ∀ s', r = .Ok s' → Inv s' ⦄ := by
  unfold transition Inv at *
  cases c <;> step*

/-- Below the limit, `Incr` succeeds and adds one. The invariant alone would
also hold for a kernel that refuses everything. -/
theorem incr_below_limit (s : State) (h : s.count.val < s.limit.val) :
    transition s .Incr ⦃ r => ∃ s', r = .Ok s' ∧ s'.count.val = s.count.val + 1 ⦄ := by
  unfold transition
  step*

/-- `Reset` always succeeds and starts again from zero. -/
theorem reset_zero (s : State) :
    transition s .Reset ⦃ r => ∃ s', r = .Ok s' ∧ s'.count.val = 0 ⦄ := by
  unfold transition
  step*

end counter_kernel
