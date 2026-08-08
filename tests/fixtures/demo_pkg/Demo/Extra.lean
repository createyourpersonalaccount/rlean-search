namespace Demo.Extra

@[simp] theorem hole_add_zero (n : Nat) : n + 0 = n := sorry

theorem named_sub_self (a : Nat) : a - a = 0 := sorry

theorem conclusion_tsum_mul (a : α) (f : β → α) : ∑' i, a * f i = a * ∑' i, f i := sorry

lemma group_mul_assoc (a b c : G) : (a * b) * c = a * (b * c) := sorry

axiom choice_ax {α : Sort u} : Nonempty α → α

end Demo.Extra
