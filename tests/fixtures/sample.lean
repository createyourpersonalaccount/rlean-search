-- Generated sample for rlean-search tests
namespace Test

theorem filter_eq_push_iff : α → Bool} {xs ys : Array α} {a : α} : filter p xs = ys.push a ↔ ∃ as bs, xs = as.push a ++ bs ∧ filter p as = ys ∧ p a ∧ (∀ x, x ∈ bs → ¬p x) := sorry

theorem eraseP_eq_iff : Array α} : xs.eraseP p = ys ↔ ((∀ a ∈ xs, ¬ p a) ∧ xs = ys) ∨ ∃ a as bs, (∀ b ∈ as, ¬ p b) ∧ p a ∧ xs = as.push a ++ bs ∧ ys = as ++ bs := sorry

lemma prod_Icc_of_even_eq_range : Type*} [CommGroup α] {f : ℤ → α} (hf : f.Even) (N : ℕ) : ∏ m ∈ Icc (-N : ℤ) N, f m = (∏ m ∈ range (N + 1), f m) ^ 2 / f 0 := sorry

theorem prod_range_induction : ℕ → M) (base : s 0 = 1) (n : ℕ) (step : ∀ k < n, s (k + 1) = s k * f k) : ∏ k ∈ Finset.range n, f k = s n := sorry

theorem prod_finsetSum_index : Finset ι} {g : ι → α →₀ M} {h : α → M → N} (h_zero : ∀ a, h a 0 = 1) (h_add : ∀ a b₁ b₂, h a (b₁ + b₂) = h a b₁ * h a b₂) : (∏ i ∈ s, (g i).prod h) = (∑ i ∈ s, g i).prod h := sorry

theorem prod_eq_mul_of_mem : Finset ι} {f : ι → M} (a b : ι) (ha : a ∈ s) (hb : b ∈ s) (hn : a ≠ b) (h₀ : ∀ c ∈ s, c ≠ a ∧ c ≠ b → f c = 1) : ∏ x ∈ s, f x = f a * f b := sorry

theorem prod_eq_mul : Finset ι} {f : ι → M} (a b : ι) (hn : a ≠ b) (h₀ : ∀ c ∈ s, c ≠ a ∧ c ≠ b → f c = 1) (ha : a ∉ s → f a = 1) (hb : b ∉ s → f b = 1) : ∏ x ∈ s, f x = f a * f b := sorry

theorem one_lt_finprod : Type*} [CommMonoid M] [PartialOrder M] [IsOrderedCancelMonoid M] {f : ι → M} (h : ∀ i, 1 ≤ f i) (h' : ∃ i, 1 < f i) (hf : HasFiniteMulSupport f) : 1 < ∏ᶠ i, f i := sorry

theorem exists_or_eq_self_of_eraseP : Array α) : xs.eraseP p = xs ∨ ∃ a ys zs, (∀ b ∈ ys, ¬p b) ∧ p a ∧ xs = ys.push a ++ zs ∧ xs.eraseP p = ys ++ zs := sorry

lemma prod_Ico_int_div : ℕ) {α : Type*} [CommGroup α] (f : ℤ → α) : ∏ n ∈ Ico (-b : ℤ) b, f n / f (n + 1) = f (-b) / f b := sorry

theorem push_eq_append_iff : Array α} {x : α} : zs.push x = xs ++ ys ↔ (ys = #[] ∧ xs = zs.push x) ∨ (∃ ys', ys = ys'.push x ∧ zs = xs ++ ys') := sorry

theorem prod_finset_product_right' : Finset (α × γ)) (s : Finset γ) (t : γ → Finset α) (h : ∀ p : α × γ, p ∈ r ↔ p.2 ∈ s ∧ p.1 ∈ t p.2) {f : α → γ → β} : ∏ p ∈ r, f p.1 p.2 = ∏ c ∈ s, ∏ a ∈ t c, f a c := sorry

theorem prod_finset_product_right : Finset (α × γ)) (s : Finset γ) (t : γ → Finset α) (h : ∀ p : α × γ, p ∈ r ↔ p.2 ∈ s ∧ p.1 ∈ t p.2) {f : α × γ → β} : ∏ p ∈ r, f p = ∏ c ∈ s, ∏ a ∈ t c, f (a, c) := sorry

theorem prod_finset_product' : Finset (γ × α)) (s : Finset γ) (t : γ → Finset α) (h : ∀ p : γ × α, p ∈ r ↔ p.1 ∈ s ∧ p.2 ∈ t p.1) {f : γ → α → β} : ∏ p ∈ r, f p.1 p.2 = ∏ c ∈ s, ∏ a ∈ t c, f c a := sorry

theorem prod_finset_product : Finset (γ × α)) (s : Finset γ) (t : γ → Finset α) (h : ∀ p : γ × α, p ∈ r ↔ p.1 ∈ s ∧ p.2 ∈ t p.1) {f : γ × α → β} : ∏ p ∈ r, f p = ∏ c ∈ s, ∏ a ∈ t c, f (c, a) := sorry

theorem prod_filter_xor : ι → Prop) [DecidablePred p] [DecidablePred q] : (∏ x ∈ s with (Xor (p x) (q x)), f x) = (∏ x ∈ s with (p x ∧ ¬ q x), f x) * (∏ x ∈ s with (q x ∧ ¬ p x), f x) := sorry

theorem prod_filter_mul_prod_filter_not : Finset ι) (p : ι → Prop) [DecidablePred p] [∀ x, Decidable (¬p x)] (f : ι → M) : (∏ x ∈ s with p x, f x) * ∏ x ∈ s with ¬p x, f x = ∏ x ∈ s, f x := sorry

theorem prod_comm' : Finset γ} {t : γ → Finset α} {t' : Finset α} {s' : α → Finset γ} (h : ∀ x y, x ∈ s ∧ y ∈ t x ↔ x ∈ s' y ∧ y ∈ t') {f : γ → α → β} : (∏ x ∈ s, ∏ y ∈ t x, f x y) = ∏ y ∈ t', ∏ x ∈ s' y, f x y := sorry

theorem mem_extract_iff_getElem : Array α} {a : α} {i j : Nat} : a ∈ as.extract i j ↔ ∃ (k : Nat) (hm : k < min j as.size - i), as[i + k] = a := sorry

theorem map_inj_of_left_inverse : α → β} (w : ∃ g : β → α, ∀ x, g (f x) = x) {x y : m α} : f <$> x = f <$> y ↔ x = y := sorry

theorem map_eq_append_iff : α → β} : map f xs = ys ++ zs ↔ ∃ as bs, xs = as ++ bs ∧ map f as = ys ∧ map f bs = zs := sorry

theorem finprod_cond_eq_prod_of_cond_iff : α → M) {p : α → Prop} {t : Finset α} (h : ∀ {x}, f x ≠ 1 → (p x ↔ x ∈ t)) : (∏ᶠ (i) (_ : p i), f i) = ∏ i ∈ t, f i := sorry

theorem findIdx_eq : α → Bool} {xs : Array α} {i : Nat} (h : i < xs.size) : xs.findIdx p = i ↔ p xs[i] ∧ ∀ j (hji : j < i), p (xs[j]'(Nat.lt_trans hji h)) = false := sorry

theorem finPiFinEquiv_apply : ℕ} {n : Fin m → ℕ} (f : ∀ i : Fin m, Fin (n i)) : (finPiFinEquiv f : ℕ) = ∑ i, f i * ∏ j, n (Fin.castLE i.is_lt.le j) := sorry

theorem filter_eq_append_iff : α → Bool} : filter p xs = ys ++ zs ↔ ∃ as bs, xs = as ++ bs ∧ filter p as = ys ∧ filter p bs = zs := sorry

theorem filterMap_eq_append_iff : α → Option β} : filterMap f xs = ys ++ zs ↔ ∃ as bs, xs = as ++ bs ∧ filterMap f as = ys ∧ filterMap f bs = zs := sorry

theorem erase_eq_iff : α} {xs : Array α} : xs.erase a = ys ↔ (a ∉ xs ∧ xs = ys) ∨ ∃ as bs, a ∉ as ∧ xs = as.push a ++ bs ∧ ys = as ++ bs := sorry

theorem eraseP_eq_empty_iff : Array α} {p : α → Bool} : xs.eraseP p = #[] ↔ xs = #[] ∨ ∃ x, p x ∧ xs = #[x] := sorry

theorem append_eq_map_iff : α → β} : xs ++ ys = map f zs ↔ ∃ as bs, zs = as ++ bs ∧ map f as = xs ∧ map f bs = ys := sorry

theorem append_eq_filterMap_iff : α → Option β} : xs ++ ys = filterMap f zs ↔ ∃ as bs, zs = as ++ bs ∧ filterMap f as = xs ∧ filterMap f bs = ys := sorry

theorem append_eq_append_iff : Array α} : ws ++ xs = ys ++ zs ↔ (∃ as, ys = ws ++ as ∧ xs = as ++ zs) ∨ ∃ cs, ws = ys ++ cs ∧ zs = cs ++ xs := sorry

theorem any_iff_exists : α → Bool} {as : Array α} {start stop} : as.any p start stop ↔ ∃ (i : Nat) (_ : i < as.size), start ≤ i ∧ i < stop ∧ p as[i] := sorry

theorem any_eq_false : α → Bool} {as : Array α} : as.any p = false ↔ ∀ (i : Nat) (_ : i < as.size), ¬p as[i] := sorry

theorem all_iff_forall : α → Bool} {as : Array α} {start stop} : as.all p start stop ↔ ∀ (i : Nat) (_ : i < as.size), start ≤ i ∧ i < stop → p as[i] := sorry

theorem all_eq_false' : α → Bool} {as : Array α} : as.all p = false ↔ ∃ x, x ∈ as ∧ ¬p x := sorry

theorem all_eq_false : α → Bool} {as : Array α} : as.all p = false ↔ ∃ (i : Nat) (_ : i < as.size), ¬p as[i] := sorry

theorem map_finprod_plift : M →* N) (g : α → M) (h : HasFiniteMulSupport <| g ∘ PLift.down) : f (∏ᶠ x, g x) = ∏ᶠ x, f (g x) := sorry

lemma sum_tsub_distrib : Finset ι) {f g : ι → M} (hfg : ∀ x ∈ s, g x ≤ f x) : ∑ x ∈ s, (f x - g x) = ∑ x ∈ s, f x - ∑ x ∈ s, g x := sorry

lemma prod_ninvolution : ι → ι) (hg₁ : ∀ a, f a * f (g a) = 1) (hg₂ : ∀ a, f a ≠ 1 → g a ≠ a) (g_mem : ∀ a, g a ∈ s) (hg₃ : ∀ a, g (g a) = a) : ∏ x ∈ s, f x = 1 := sorry

lemma prod_involution : ∀ a ∈ s, ι) (hg₁ : ∀ a ha, f a * f (g a ha) = 1) (hg₃ : ∀ a ha, f a ≠ 1 → g a ha ≠ a) (g_mem : ∀ a ha, g a ha ∈ s) (hg₄ : ∀ a ha, g (g a ha) (g_mem a ha) = a) : ∏ x ∈ s, f x = 1 := sorry

lemma prod_filter_not_mul_prod_filter : Finset ι) (p : ι → Prop) [DecidablePred p] [∀ x, Decidable (¬p x)] (f : ι → M) : (∏ x ∈ s with ¬p x, f x) * ∏ x ∈ s with p x, f x = ∏ x ∈ s, f x := sorry

lemma prod_eq_prod_iff_single : ι → M} {i : ι} (hi : i ∈ s) (hfg : ∀ j ∈ s, j ≠ i → f j = g j) : ∏ j ∈ s, f j = ∏ j ∈ s, g j ↔ f i = g i := sorry

lemma prod_Icc_succ_eq_mul_endpoints : Type*} [CommGroup R] (f : ℤ → R) {N : ℕ} : ∏ m ∈ Icc (-(N + 1) : ℤ) (N + 1), f m = f (N + 1) * f (-(N + 1) : ℤ) * ∏ m ∈ Icc (-N : ℤ) N, f m := sorry

lemma eq_prod_range_div : ℕ → G) (n : ℕ) : f n = f 0 * ∏ i ∈ range n, f (i + 1) / f i := sorry

theorem toRingHom_toOpposite : A →ₐ[R] B) (hf : ∀ x y, Commute (f x) (f y)) : (f.toOpposite hf : A →+* Bᵐᵒᵖ) = (f : A →+* B).toOpposite hf := sorry

theorem toRingHom_fromOpposite : A →ₐ[R] B) (hf : ∀ x y, Commute (f x) (f y)) : (f.fromOpposite hf : Aᵐᵒᵖ →+* B) = (f : A →+* B).fromOpposite hf := sorry

theorem sum_const_nat : ℕ} {f : ι → ℕ} (h₁ : ∀ x ∈ s, f x = m) : ∑ x ∈ s, f x = #s * m := sorry

theorem size_filter_lt_size_iff_exists : Array α} {p : α → Bool} : (filter p xs).size < xs.size ↔ ∃ x ∈ xs, ¬p x := sorry

theorem size_filterMap_pos_iff : Array α} {f : α → Option β} : 0 < (filterMap f xs).size ↔ ∃ (x : α) (_ : x ∈ xs) (b : β), f x = some b := sorry

theorem size_filterMap_lt_size_iff_exists : Array α} {f : α → Option β} : (filterMap f xs).size < xs.size ↔ ∃ (x : α) (_ : x ∈ xs), f x = none := sorry

theorem singleton_eq_append_iff : Array α} {x : α} : #[x] = xs ++ ys ↔ (xs = #[] ∧ ys = #[x]) ∨ (xs = #[x] ∧ ys = #[]) := sorry

theorem single_le_finprod : Type*} [CommMonoid M] [Preorder M] [IsOrderedMonoid M] (i : α) {f : α → M} (hf : HasFiniteMulSupport f) (h : ∀ j, 1 ≤ f j) : f i ≤ ∏ᶠ j, f j := sorry

theorem rel_of_isEqv : α → α → Bool} {xs ys : Array α} : Array.isEqv xs ys r → ∃ h : xs.size = ys.size, ∀ (i : Nat) (h' : i < xs.size), r (xs[i]) (ys[i]'(h ▸ h')) := sorry

theorem prod_univ_succ : Fin (n + 1) → M) : ∏ i, f i = f 0 * ∏ i : Fin n, f i.succ := sorry

theorem prod_univ_castSucc : Fin (n + 1) → M) : ∏ i, f i = (∏ i : Fin n, f (Fin.castSucc i)) * f (last n) := sorry

theorem prod_univ_add : ℕ} (f : Fin (a + b) → M) : (∏ i : Fin (a + b), f i) = (∏ i : Fin a, f (castAdd b i)) * ∏ i : Fin b, f (natAdd a i) := sorry

theorem prod_trunc : ℕ} (f : Fin (a + b) → M) (hf : ∀ j : Fin b, f (natAdd a j) = 1) : (∏ i : Fin (a + b), f i) = ∏ i : Fin a, f (castAdd b i) := sorry

theorem prod_sum_index : α →₀ M} {g : α → M → β →₀ N} {h : β → N → P} (h_zero : ∀ a, h a 0 = 1) (h_add : ∀ a b₁ b₂, h a (b₁ + b₂) = h a b₁ * h a b₂) : (f.sum g).prod h = f.prod fun a b => (g a b).prod h := sorry

theorem prod_sum : Type*} [CommMonoid M] (f : ι → Multiset M) (s : Finset ι) : (∑ x ∈ s, f x).prod = ∏ x ∈ s, (f x).prod := sorry

theorem prod_subtype_map_embedding : ι → Prop} {s : Finset { x // p x }} {f : { x // p x } → M} {g : ι → M} (h : ∀ x : { x // p x }, x ∈ s → g x = f x) : (∏ x ∈ s.map (Function.Embedding.subtype _), g x) = ∏ x ∈ s, f x := sorry

theorem prod_sigma' : α → Type*} (s : Finset α) (t : ∀ a, Finset (σ a)) (f : ∀ a, σ a → β) : (∏ a ∈ s, ∏ s ∈ t a, f a s) = ∏ x ∈ s.sigma t, f x.1 x.2 := sorry

theorem prod_sigma : α → Type*} (s : Finset α) (t : ∀ a, Finset (σ a)) (f : Sigma σ → β) : ∏ x ∈ s.sigma t, f x = ∏ a ∈ s, ∏ s ∈ t a, f ⟨a, s⟩ := sorry

theorem prod_range_succ : ℕ → M) (n : ℕ) : (∏ x ∈ range (n + 1), f x) = (∏ x ∈ range n, f x) * f n := sorry

theorem prod_range_add : ℕ → M) (n m : ℕ) : (∏ x ∈ range (n + m), f x) = (∏ x ∈ range n, f x) * ∏ x ∈ range m, f (n + x) := sorry

theorem prod_prod_eq_prod_triangle_mul : Fin (n + 1) → Fin n → M) : ∏ i, ∏ j, f i j = ∏ i : Fin n, ∏ j ≥ i, (f i.castSucc j * f j.succ i) := sorry

theorem prod_pi_mulSingle : ι → Type*} [DecidableEq ι] [∀ a, CommMonoid (M a)] (a : ι) (f : ∀ a, M a) (s : Finset ι) : (∏ a' ∈ s, Pi.mulSingle a' (f a') a) = if a ∈ s then f a else 1 := sorry

theorem prod_multiset_map_count : Multiset ι) {M : Type*} [CommMonoid M] (f : ι → M) : (s.map f).prod = ∏ m ∈ s.toFinset, f m ^ s.count m := sorry

theorem prod_mem_multiset : Multiset ι) (f : { x // x ∈ m } → M) (g : ι → M) (hfg : ∀ x, f x = g x) : ∏ x : { x // x ∈ m }, f x = ∏ x ∈ m.toFinset, g x := sorry

theorem prod_map_eq_pow_single : ι) (hf : ∀ i' ≠ i, i' ∈ m → f i' = 1) : (m.map f).prod = f i ^ m.count i := sorry

theorem prod_ite_one : Finset ι) (p : ι → Prop) [DecidablePred p] (h : ∀ i ∈ s, ∀ j ∈ s, p i → p j → i = j) (a : M) : ∏ i ∈ s, ite (p i) a 1 = ite (∃ i ∈ s, p i) a 1 := sorry

theorem prod_ite : Finset ι} {p : ι → Prop} [DecidablePred p] (f g : ι → M) : ∏ x ∈ s, (if p x then f x else g x) = (∏ x ∈ s with p x, f x) * ∏ x ∈ s with ¬p x, g x := sorry

theorem prod_induction_nonempty : Type*} [CommMonoid M] (f : ι → M) (p : M → Prop) (hom : ∀ a b, p a → p b → p (a * b)) (nonempty : s.Nonempty) (base : ∀ x ∈ s, p <| f x) : p <| ∏ x ∈ s, f x := sorry

theorem prod_induction : Type*} [CommMonoid M] (f : ι → M) (p : M → Prop) (hom : ∀ a b, p a → p b → p (a * b)) (unit : p 1) (base : ∀ x ∈ s, p <| f x) : p <| ∏ x ∈ s, f x := sorry

theorem prod_flip : ℕ} (f : ℕ → M) : (∏ r ∈ range (n + 1), f (n - r)) = ∏ k ∈ range (n + 1), f k := sorry

theorem prod_filter_of_ne : ι → Prop} [DecidablePred p] (hp : ∀ x ∈ s, f x ≠ 1 → p x) : ∏ x ∈ s with p x, f x = ∏ x ∈ s, f x := sorry

theorem prod_eq_single_of_mem : Finset ι} {f : ι → M} (a : ι) (h : a ∈ s) (h₀ : ∀ b ∈ s, b ≠ a → f b = 1) : ∏ x ∈ s, f x = f a := sorry

theorem prod_eq_single : Finset ι} {f : ι → M} (a : ι) (h₀ : ∀ b ∈ s, b ≠ a → f b = 1) (h₁ : a ∉ s → f a = 1) : ∏ x ∈ s, f x = f a := sorry

theorem prod_eq_pow_single : M) (h : ∀ a' ≠ a, a' ∈ s → a' = 1) : s.prod = a ^ s.count a := sorry

theorem prod_bij : ∀ a ∈ s, κ) (hi : ∀ a ha, i a ha ∈ t) (i_inj : ∀ a₁ ha₁ a₂ ha₂, i a₁ ha₁ = i a₂ ha₂ → a₁ = a₂) (i_surj : ∀ b ∈ t, ∃ a ha, i a ha = b) (h : ∀ a ha, f a = g (i a ha)) : ∏ x ∈ s, f x = ∏ x ∈ t, g x := sorry

theorem prod_apply_ite_of_false : ι → Prop} [DecidablePred p] (f g : ι → γ) (k : γ → M) (h : ∀ x ∈ s, ¬p x) : (∏ x ∈ s, k (if p x then f x else g x)) = ∏ x ∈ s, k (g x) := sorry

end Test
