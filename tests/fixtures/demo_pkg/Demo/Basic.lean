-- Demo package for rlean-search tests
namespace Demo

theorem add_comm : ∀ (n m : Nat), n + m = m + n := sorry

theorem add_assoc : ∀ (n m k : Nat), (n + m) + k = n + (m + k) := sorry

theorem add_zero : ∀ (n : Nat), n + 0 = n := sorry

theorem zero_add : ∀ (n : Nat), 0 + n = n := sorry

theorem mul_comm : ∀ (n m : Nat), n * m = m * n := sorry

theorem mul_assoc : ∀ (n m k : Nat), (n * m) * k = n * (m * k) := sorry

theorem mul_one : ∀ (n : Nat), n * 1 = n := sorry

theorem one_mul : ∀ (n : Nat), 1 * n = n := sorry

theorem mul_zero : ∀ (n : Nat), n * 0 = 0 := sorry

theorem zero_mul : ∀ (n : Nat), 0 * n = 0 := sorry

theorem add_left_cancel : ∀ {n m k : Nat}, n + m = n + k → m = k := sorry

theorem eq_zero_of_add_eq_zero : ∀ {n m : Nat}, n + m = 0 → n = 0 ∧ m = 0 := sorry

theorem succ_add : ∀ (n m : Nat), (n + 1) + m = (n + m) + 1 := sorry

theorem add_succ : ∀ (n m : Nat), n + (m + 1) = (n + m) + 1 := sorry

theorem le_refl : ∀ (n : Nat), n ≤ n := sorry

theorem le_trans : ∀ {n m k : Nat}, n ≤ m → m ≤ k → n ≤ k := sorry

theorem lt_irrefl : ∀ (n : Nat), ¬ n < n := sorry

theorem not_lt_zero : ∀ (n : Nat), ¬ n < 0 := sorry

theorem zero_le : ∀ (n : Nat), 0 ≤ n := sorry

theorem succ_le_succ : ∀ {n m : Nat}, n ≤ m → n + 1 ≤ m + 1 := sorry

theorem sub_self : ∀ (n : Nat), n - n = 0 := sorry

theorem zero_sub : ∀ (n : Nat), 0 - n = 0 := sorry

theorem sub_zero : ∀ (n : Nat), n - 0 = n := sorry

theorem add_sub_cancel : ∀ (n m : Nat), n + m - m = n := sorry

theorem and_comm : ∀ {a b : Prop}, a ∧ b ↔ b ∧ a := sorry

theorem or_comm : ∀ {a b : Prop}, a ∨ b ↔ b ∨ a := sorry

theorem not_not : ∀ {a : Prop}, ¬¬a ↔ a := sorry

theorem iff_refl : ∀ (a : Prop), a ↔ a := sorry

theorem Eq.trans : ∀ {α : Sort u} {a b c : α}, a = b → b = c → a = c := sorry

theorem Eq.symm : ∀ {α : Sort u} {a b : α}, a = b → b = a := sorry

theorem congrArg : ∀ {α : Sort u} {β : Sort v} {a₁ a₂ : α} (f : α → β), a₁ = a₂ → f a₁ = f a₂ := sorry

theorem congr : ∀ {α : Sort u} {β : Sort v} {f₁ f₂ : α → β} {a₁ a₂ : α}, f₁ = f₂ → a₁ = a₂ → f₁ a₁ = f₂ a₂ := sorry

axiom sorryAx : ∀ (α : Sort u) (synthetic : Bool), α := sorry

axiom Classical.choice : ∀ {α : Sort u}, Nonempty α → α := sorry

theorem mul_left_comm : ∀ (a b c : G), a * (b * c) = b * (a * c) := sorry

theorem mul_right_comm : ∀ (a b c : G), a * b * c = a * c * b := sorry

theorem one_mul_eq_id : ((1 : M) * ·) = id := sorry

theorem mul_one_eq_id : (· * (1 : M)) = id := sorry

theorem inv_involutive : Function.Involutive (Inv.inv : G → G) := sorry

theorem mul_eq_left : ∀ {a b : M}, a * b = a ↔ b = 1 := sorry

theorem mul_eq_right : ∀ {a b : M}, a * b = b ↔ a = 1 := sorry

lemma pow_add : ∀ (a : M) (m n : ℕ), a ^ (m + n) = a ^ m * a ^ n := sorry

lemma pow_one : ∀ (a : M), a ^ 1 = a := sorry

lemma pow_zero : ∀ (a : M), a ^ 0 = 1 := sorry

lemma one_pow : ∀ (n : ℕ), (1 : M) ^ n = 1 := sorry

theorem tsum_add : ∑' i, (f i + g i) = ∑' i, f i + ∑' i, g i := sorry

theorem tsum_mul_left : ∑' i, a * f i = a * ∑' i, f i := sorry

theorem tsum_const_smul : ∑' i, c • f i = c • ∑' i, f i := sorry

theorem list_length_append : ∀ (as bs : List α), (as ++ bs).length = as.length + bs.length := sorry

theorem list_length_map : ∀ (f : α → β) (as : List α), (as.map f).length = as.length := sorry

theorem list_nil_append : ∀ (as : List α), [] ++ as = as := sorry

theorem list_append_nil : ∀ (as : List α), as ++ [] = as := sorry

theorem list_append_assoc : ∀ (as bs cs : List α), (as ++ bs) ++ cs = as ++ (bs ++ cs) := sorry

theorem option_map_id : ∀ (x : Option α), x.map id = x := sorry

theorem option_map_map : ∀ (f : β → γ) (g : α → β) (x : Option α), (x.map g).map f = x.map (f ∘ g) := sorry

theorem nat_lt_succ_self : ∀ (n : Nat), n < n + 1 := sorry

theorem nat_le_add_right : ∀ (n k : Nat), n ≤ n + k := sorry

theorem nat_le_add_left : ∀ (n k : Nat), n ≤ k + n := sorry

theorem div_self : ∀ {n : Nat}, n ≠ 0 → n / n = 1 := sorry

theorem mod_self : ∀ (n : Nat), n % n = 0 := sorry

theorem add_mod : ∀ (a b n : Nat), (a + b) % n = (a % n + b % n) % n := sorry

theorem mul_mod : ∀ (a b n : Nat), (a * b) % n = (a % n * b % n) % n := sorry

theorem dvd_refl : ∀ (a : Nat), a ∣ a := sorry

theorem dvd_trans : ∀ {a b c : Nat}, a ∣ b → b ∣ c → a ∣ c := sorry

theorem dvd_mul_right : ∀ (a b : Nat), a ∣ a * b := sorry

theorem dvd_mul_left : ∀ (a b : Nat), a ∣ b * a := sorry

theorem and_true : ∀ (p : Prop), p ∧ True ↔ p := sorry

theorem true_and : ∀ (p : Prop), True ∧ p ↔ p := sorry

theorem or_false : ∀ (p : Prop), p ∨ False ↔ p := sorry

theorem false_or : ∀ (p : Prop), False ∨ p ↔ p := sorry

theorem imp_self : ∀ (p : Prop), (p → p) := sorry

theorem not_and : ∀ (p q : Prop), ¬(p ∧ q) ↔ ¬p ∨ ¬q := sorry

theorem not_or : ∀ (p q : Prop), ¬(p ∨ q) ↔ ¬p ∧ ¬q := sorry

theorem forall_const : ∀ (α : Sort u) (p : Prop), (α → p) ↔ (Nonempty α → p) := sorry

theorem exists_const : ∀ (α : Sort u) [Nonempty α] (p : Prop), (∃ x : α, p) ↔ p := sorry

theorem Function.comp_assoc : ∀ {α β γ δ} (f : γ → δ) (g : β → γ) (h : α → β), (f ∘ g) ∘ h = f ∘ g ∘ h := sorry

theorem id_comp : ∀ {α β} (f : α → β), id ∘ f = f := sorry

theorem comp_id : ∀ {α β} (f : α → β), f ∘ id = f := sorry

lemma smul_add : ∀ (r : R) (x y : M), r • (x + y) = r • x + r • y := sorry

lemma add_smul : ∀ (r s : R) (x : M), (r + s) • x = r • x + s • x := sorry

lemma one_smul : ∀ (x : M), (1 : R) • x = x := sorry

lemma zero_smul : ∀ (x : M), (0 : R) • x = 0 := sorry

lemma smul_zero : ∀ (r : R), r • (0 : M) = 0 := sorry

theorem neg_neg : ∀ (a : G), - -a = a := sorry

theorem neg_add : ∀ (a b : G), -(a + b) = -b + -a := sorry

theorem add_left_neg : ∀ (a : G), -a + a = 0 := sorry

theorem add_right_neg : ∀ (a : G), a + -a = 0 := sorry

theorem sub_eq_add_neg : ∀ (a b : G), a - b = a + -b := sorry

theorem sub_self_eq : ∀ (a : G), a - a = 0 := sorry

theorem add_sub_cancel_right : ∀ (a b : G), a + b - b = a := sorry

theorem tsum_eq_zero : ∑' i, f i = 0 := sorry

theorem tsum_mul_tsum : ∑' i, f i * ∑' j, g j = ∑' i, ∑' j, f i * g j := sorry

theorem finset_sum_add : ∑ i ∈ s, (f i + g i) = ∑ i ∈ s, f i + ∑ i ∈ s, g i := sorry

theorem finset_sum_mul_left : ∑ i ∈ s, a * f i = a * ∑ i ∈ s, f i := sorry

theorem prod_mul : ∏ i ∈ s, (f i * g i) = (∏ i ∈ s, f i) * (∏ i ∈ s, g i) := sorry

theorem abs_neg : ∀ (a : ℝ), | -a | = |a| := sorry

theorem abs_mul : ∀ (a b : ℝ), |a * b| = |a| * |b| := sorry

theorem abs_add_le : ∀ (a b : ℝ), |a + b| ≤ |a| + |b| := sorry

theorem le_antisymm : ∀ {a b : α}, a ≤ b → b ≤ a → a = b := sorry

theorem lt_asymm : ∀ {a b : α}, a < b → ¬ b < a := sorry

theorem ne_of_lt : ∀ {a b : α}, a < b → a ≠ b := sorry

theorem ne_of_gt : ∀ {a b : α}, a > b → a ≠ b := sorry

theorem max_comm : ∀ (a b : α), max a b = max b a := sorry

theorem min_comm : ∀ (a b : α), min a b = min b a := sorry

theorem max_self : ∀ (a : α), max a a = a := sorry

theorem min_self : ∀ (a : α), min a a = a := sorry

end Demo