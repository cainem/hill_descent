# M6 Gaussian Noise Mutation - Comprehensive Implementation Plan

## Overview
Add two new evolvable system parameters (m6 and m6_sigma) to implement Gaussian noise mutation on `adjustment_value` in `LocusAdjustment`. This provides fine-grained local search capability to complement the existing coarse-grained exponential mutations (m5 doubling/halving).

### Goals
- Add m6 (probability of Gaussian noise mutation) as evolvable parameter
- Add m6_sigma (scale of Gaussian noise as proportion) as evolvable parameter  
- Implement Gaussian noise mutation in locus mutation logic
- Ensure all tests pass with updated parameter counts
- Update PDD documentation
- Update server and UI to display new parameters

---

## Implementation Tasks

### PHASE 1: Core Library Changes

#### 1.1 Update SystemParameters Structure
**File:** `hill_descent_lib/src/parameters/system_parameters.rs`

**Changes Required:**
- Add two new fields to `SystemParameters` struct:
  - `m6: f64` (probability of noise mutation)
  - `m6_sigma: f64` (scale of noise as proportion)
- Update `SystemParameters::new()` to expect 9 elements (was 7)
- Add getter methods: `m6()` and `m6_sigma()`
- Update all documentation comments describing parameter order
- Update panic messages for incorrect slice lengths

**Test Updates:**
- Update `given_correct_length_slice_when_new_then_all_fields_are_set_correctly` to use 9 elements
- Update all panic test expected messages from "7 elements" to "9 elements"
- Add assertions for `m6()` and `m6_sigma()` getters

**Example:**
```rust
pub struct SystemParameters {
    m1: f64,
    m2: f64,
    m3: f64,
    m4: f64,
    m5: f64,
    m6: f64,           // NEW: Probability of Gaussian noise mutation
    m6_sigma: f64,     // NEW: Scale of Gaussian noise (proportion)
    max_age: f64,
    crossover_points: f64,
}

pub fn new(values: &[f64]) -> Self {
    if values.len() != 9 {  // Changed from 7
        panic!("SystemParameters::new expects a slice with exactly 9 elements, got {}", values.len());
    }
    Self {
        m1: values[0],
        m2: values[1],
        m3: values[2],
        m4: values[3],
        m5: values[4],
        m6: values[5],        // NEW
        m6_sigma: values[6],  // NEW
        max_age: values[7],   // Shifted from index 5
        crossover_points: values[8],  // Shifted from index 6
    }
}

pub fn m6(&self) -> f64 {
    self.m6
}

pub fn m6_sigma(&self) -> f64 {
    self.m6_sigma
}
```

---

#### 1.2 Update Parameter Enhancement
**File:** `hill_descent_lib/src/parameters/parameter_enhancement.rs`

**Changes Required:**
- Add two new parameters to `system_params_to_prepend` array
- Update array size and extend documentation

**Configuration:**
```rust
let system_params_to_prepend = [
    Parameter::with_bounds(0.1, 0.0, 1.0),     // m1_prob_false_to_true
    Parameter::with_bounds(0.5, 0.0, 1.0),     // m2_prob_true_to_false
    Parameter::with_bounds(0.001, 0.0, 1.0),   // m3_prob_adj_double_halve_flag
    Parameter::with_bounds(0.001, 0.0, 1.0),   // m4_prob_adj_direction_flag
    Parameter::with_bounds(0.001, 0.0, 1.0),   // m5_prob_locus_value_mutation
    Parameter::with_bounds(0.01, 0.0, 1.0),    // NEW: m6_prob_gaussian_noise (1% initial)
    Parameter::with_bounds(0.1, 0.001, 1.0),   // NEW: m6_sigma (10% initial, min 0.1%)
    Parameter::with_bounds(5.0, 2.0, 10.0),    // max_age
    Parameter::with_bounds(2.0, 1.0, 10.0),    // crossover_points
];
```

**Test Updates:**
- Update `given_empty_slice_when_enhance_parameters_called_then_returns_only_system_parameter_bounds` to assert length == 9 (was 7)
- Update `given_non_empty_slice_when_enhance_parameters_called_then_prepends_system_params` to use 9 system params
- Add test for m6 and m6_sigma bounds verification

---

#### 1.3 Implement Gaussian Noise Mutation
**File:** `hill_descent_lib/src/locus/mutate.rs`

**Changes Required:**
- Add `use rand_distr::{Distribution, StandardNormal};` at top of file
- Add Gaussian noise mutation after existing m5 mutation in both `mutate()` and `mutate_unbound()`
- Apply noise probabilistically based on m6
- Scale noise by m6_sigma proportion

**Implementation Location:**
Insert after m5 mutation (after line ~37 in `mutate()` and after line ~91 in `mutate_unbound()`):

```rust
// Gaussian noise mutation (m6) - applies after m5
// This adds proportional random noise for fine-grained local search
if rng.random_bool(sys.m6()) {
    let current_value = new_adj_val.get();
    let noise_scale = sys.m6_sigma();
    let noise = StandardNormal.sample(rng) * current_value * noise_scale;
    let candidate_value = current_value + noise;
    
    // Only apply if result is finite and non-negative (adjustment_value must be >= 0)
    if candidate_value.is_finite() && candidate_value >= 0.0 {
        new_adj_val.set(candidate_value);
    }
    // else: mutation fails silently, keep original value
}
```

**Important Notes:**
- Apply m6 AFTER m5 (so noise can refine the doubled/halved value)
- Validate that result is finite and >= 0 (LocusAdjustment requirement)
- Rejection is silent (keeps original value) - this is acceptable since unbounded values rarely reject
- Same logic applies to both `mutate()` and `mutate_unbound()`

**Test Updates:**
- Update ALL existing test SystemParameters::new() calls from 7 to 9 elements
- Add two new elements: m6 probability (usually 0.0) and m6_sigma (e.g., 0.1)
- Add new tests:
  - `given_m6_one_when_mutate_then_gaussian_noise_applied`
  - `given_m6_zero_when_mutate_then_no_gaussian_noise_applied`
  - `given_m6_noise_creates_negative_when_mutate_then_value_unchanged`
  - `given_m6_sigma_when_mutate_then_noise_scaled_proportionally`

**Example Test Pattern:**
```rust
#[test]
fn given_m6_one_when_mutate_then_gaussian_noise_applied() {
    let mut rng = StdRng::from_seed([42; 32]);
    let sys = SystemParameters::new(&[0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.5, 0.0, 0.0]); 
    // m6=1.0 (always mutate), m6_sigma=0.5 (50% noise)
    
    let adj_param = Parameter::with_bounds(10.0, 0.0, 100.0);
    let adj = LocusAdjustment::new(adj_param, DirectionOfTravel::Add, false);
    let initial_locus = Locus::new(Parameter::new(5.0), adj, false);
    
    let mutated = initial_locus.mutate(&mut rng, &sys);
    
    // Adjustment value should have changed due to Gaussian noise
    assert_ne!(
        mutated.adjustment().adjustment_value().get(),
        10.0,
        "Gaussian noise should have changed adjustment_value"
    );
}
```

---

#### 1.4 Update Phenotype NUM_SYSTEM_PARAMETERS Constant
**File:** `hill_descent_lib/src/phenotype/mod.rs`

**Changes Required:**
- Update `NUM_SYSTEM_PARAMETERS` constant from 7 to 9
- Update related documentation

```rust
/// Number of system-wide evolvable parameters (m1, m2, m3, m4, m5, m6, m6_sigma, max_age, crossover_points)
pub const NUM_SYSTEM_PARAMETERS: usize = 9;  // Changed from 7
```

---

#### 1.5 Update World State Serialization for Web
**File:** `hill_descent_lib/src/world/get_state_for_web.rs`

**Changes Required:**
- Add `m6` and `m6_sigma` fields to `SystemParametersState` struct
- Update `from_system_parameters()` to include new fields

```rust
#[derive(Serialize, Debug)]
struct SystemParametersState {
    m1: f64,
    m2: f64,
    m3: f64,
    m4: f64,
    m5: f64,
    m6: f64,           // NEW
    m6_sigma: f64,     // NEW
    max_age: f64,
    crossover_points: f64,
}

impl SystemParametersState {
    fn from_system_parameters(
        sys_params: &crate::parameters::system_parameters::SystemParameters,
    ) -> Self {
        Self {
            m1: sys_params.m1(),
            m2: sys_params.m2(),
            m3: sys_params.m3(),
            m4: sys_params.m4(),
            m5: sys_params.m5(),
            m6: sys_params.m6(),           // NEW
            m6_sigma: sys_params.m6_sigma(),  // NEW
            max_age: sys_params.max_age(),
            crossover_points: sys_params.crossover_points(),
        }
    }
}
```

**Test Updates:**
- Verify JSON output includes m6 and m6_sigma fields in organism phenotype data

---

#### 1.6 Update All Test Files with SystemParameters::new() Calls
**Files to Update:** (Search for all `SystemParameters::new` calls)
- `hill_descent_lib/src/locus/mutate.rs` - Update all test calls (20+ instances)
- `hill_descent_lib/src/gamete/reproduce.rs` - Update test calls
- `hill_descent_lib/src/phenotype/sexual_reproduction.rs` - Update test calls
- Any other test files using SystemParameters

**Pattern:**
Change from:
```rust
let sys = SystemParameters::new(&[0.1, 0.5, 0.001, 0.001, 0.001, 100.0, 2.0]);
```

To:
```rust
let sys = SystemParameters::new(&[0.1, 0.5, 0.001, 0.001, 0.001, 0.01, 0.1, 100.0, 2.0]);
//                                  m1   m2   m3    m4    m5    m6   σ    age  xover
```

**Recommended approach:**
- Set m6=0.0 and m6_sigma=0.1 for most existing tests (no behavior change)
- Only set m6=1.0 in new specific m6 tests

---

### PHASE 2: Documentation Updates

#### 2.1 Update Product Definition Document (PDD)
**File:** `hill_descent_lib/pdd.md`

**Changes Required:**

**Section 4.1.3.b (LocusAdjustment properties):**
- No changes needed (m6 doesn't affect the structure)

**Section 5.2.4 (Mutation section):**
Add new subsection **5.2.4.6** after existing mutation descriptions:

```markdown
* **5.2.4.6. Adjustment `AdjustmentValue` Gaussian Noise Mutation:** 
    * Probability: $m_6$ (e.g., 1%).  
    * Action: Adds Gaussian-distributed random noise proportional to current `AdjustmentValue`.
    * Noise Scale: Controlled by $m_{6\sigma}$ parameter (e.g., 0.1 = 10% of current value).
    * Noise is drawn from $N(0, \sigma^2)$ where $\sigma = \text{AdjustmentValue} \times m_{6\sigma}$.
    * If the resulting value is negative or non-finite, the mutation is rejected and the original value retained.
    * This mutation provides fine-grained local search capability, complementing the coarse exponential changes from m5.
    * Both $m_6$ and $m_{6\sigma}$ are evolvable parameters with their own bounds.
```

**Section 4.1.3.a (System Parameters list):**
Update the enumeration to include m6 and m6_sigma in the list of evolvable mutation parameters.

---

### PHASE 3: Server and UI Updates

#### 3.1 Update Web UI JavaScript
**File:** `hill_descent_server/web/main.js`

**Changes Required:**

**In organism detail display function** (search for SystemParametersState rendering):
Add display of m6 and m6_sigma parameters alongside existing m1-m5:

```javascript
// In the function that displays system parameters (around line 800-900)
// Add after m5 display:
<div class="param-row">
    <span class="param-label">m6 (Gaussian Noise Prob):</span>
    <span class="param-value">${NumberFormatter.format(sysParams.m6, 4)}</span>
</div>
<div class="param-row">
    <span class="param-label">m6_sigma (Noise Scale):</span>
    <span class="param-value">${NumberFormatter.format(sysParams.m6_sigma, 4)}</span>
</div>
```

**Location:** Find where m1, m2, m3, m4, m5 are displayed in organism panels and add m6 and m6_sigma in the same style.

---

#### 3.2 Update Web PDD Documentation
**File:** `hill_descent_server/web/web_pdd.md`

**Changes Required:**
Update the `SystemParametersState` structure documentation to include m6 and m6_sigma:

```markdown
### SystemParametersState
- `m1`: float - Mutation probability (ApplyAdjustmentFlag: False -> True)
- `m2`: float - Mutation probability (ApplyAdjustmentFlag: True -> False)  
- `m3`: float - Mutation probability (LocusAdjustment.DoublingOrHalvingFlag)
- `m4`: float - Mutation probability (LocusAdjustment.DirectionOfTravel)
- `m5`: float - Mutation probability (LocusValue via doubling/halving)
- `m6`: float - Mutation probability (Gaussian noise on AdjustmentValue) **NEW**
- `m6_sigma`: float - Scale of Gaussian noise as proportion of value **NEW**
- `max_age`: float - Maximum organism age
- `crossover_points`: float - Number of crossover points for reproduction
```

---

### PHASE 4: Integration Testing

#### 4.1 Run Full Test Suite
**Command:** `cargo test --workspace`

**Expected Results:**
- All existing tests should pass after parameter count updates
- New m6-specific tests should pass
- Integration tests should work with 9 parameters

#### 4.2 Run Benchmarks
**Command:** `cargo bench`

**Expected Results:**
- Performance should be minimally impacted (one additional random number generation when m6 triggers)
- Verify that m6=0.0 has zero overhead

#### 4.3 Manual Testing with Server
**Steps:**
1. Start server: `cargo run` (from hill_descent_server/)
2. Open browser to http://localhost:3000
3. Start optimization with any function
4. Click on organisms to view details
5. Verify m6 and m6_sigma are displayed in organism panels
6. Verify values are within expected bounds (m6: 0-1, m6_sigma: 0.001-1)

---

### PHASE 5: Validation and Cleanup

#### 5.1 Code Quality Checks
**Commands:**
```powershell
cargo fmt              # Format all code
cargo clippy           # Lint checks
cargo clippy --tests   # Lint test code
```

**Expected Results:**
- No formatting issues
- No clippy warnings related to new code
- All code follows project standards

#### 5.2 Documentation Review
**Checklist:**
- [ ] All new functions have doc comments
- [ ] PDD is updated and accurate
- [ ] web_pdd.md reflects new structure
- [ ] AGENTS.md is still accurate (no changes needed)
- [ ] .github/copilot-instructions.md is still accurate (no changes needed)

#### 5.3 Test Coverage Verification
**Checklist:**
- [ ] m6=0.0 produces no mutations (backward compatibility)
- [ ] m6=1.0 always produces mutations
- [ ] m6_sigma scales noise appropriately
- [ ] Negative results are rejected
- [ ] Non-finite results are rejected
- [ ] All existing tests pass with 9 parameters
- [ ] New tests cover all m6 branches and conditions

---

## Implementation Order

**Recommended sequence for implementing agent:**

1. **Start with SystemParameters** (1.1) - Foundation for everything else
2. **Update parameter_enhancement** (1.2) - Ensures proper initialization
3. **Update NUM_SYSTEM_PARAMETERS** (1.4) - Required before phenotype tests work
4. **Update all test SystemParameters::new() calls** (1.6) - Makes existing tests pass
5. **Implement Gaussian mutation logic** (1.3) - Core functionality + new tests
6. **Update web state serialization** (1.5) - For server integration
7. **Update PDD documentation** (2.1) - Document the changes
8. **Update UI and web PDD** (3.1, 3.2) - Frontend display
9. **Run integration tests** (4.1, 4.2, 4.3) - Validate everything works
10. **Final cleanup** (5.1, 5.2, 5.3) - Polish and verify

---

## Expected Outcomes

### Performance Impact
- Minimal overhead when m6=0.0 (one probability check)
- When m6>0, adds one random normal sample per mutation
- Expected: <5% performance impact even with m6=1.0

### Behavioral Impact
- Bukin N6 performance should improve significantly (goal: scores closer to 0.0)
- Other functions may see moderate improvements
- High variability initially as populations explore different m6/m6_sigma values
- Over time, successful values should emerge for each problem type

### Evolvability
- Populations will naturally tune m6 and m6_sigma to problem characteristics
- Early generations may have diverse values (exploration)
- Convergence should see smaller m6_sigma values (exploitation)
- Different lineages may adopt different strategies (diversity)

---

## Files Summary

### Core Library Files to Modify
1. `hill_descent_lib/src/parameters/system_parameters.rs` - Add m6, m6_sigma fields and getters
2. `hill_descent_lib/src/parameters/parameter_enhancement.rs` - Add m6, m6_sigma bounds
3. `hill_descent_lib/src/locus/mutate.rs` - Implement Gaussian mutation logic
4. `hill_descent_lib/src/phenotype/mod.rs` - Update NUM_SYSTEM_PARAMETERS constant
5. `hill_descent_lib/src/world/get_state_for_web.rs` - Add m6, m6_sigma to serialization
6. `hill_descent_lib/src/gamete/reproduce.rs` - Update test SystemParameters calls
7. `hill_descent_lib/src/phenotype/sexual_reproduction.rs` - Update test SystemParameters calls

### Documentation Files to Modify
8. `hill_descent_lib/pdd.md` - Add section 5.2.4.6 and update parameter lists
9. `hill_descent_server/web/web_pdd.md` - Update SystemParametersState documentation

### UI Files to Modify
10. `hill_descent_server/web/main.js` - Add m6, m6_sigma display in organism panels

### No Changes Required
- `AGENTS.md` - Still accurate
- `.github/copilot-instructions.md` - Still accurate
- `hill_descent_server/src/main.rs` - No server logic changes needed
- All test function implementations - Just parameter updates

---

## Testing Strategy

### Unit Tests (per function)
- Test m6=0.0 → no mutation
- Test m6=1.0 → always mutates
- Test m6_sigma scales proportionally
- Test negative rejection
- Test non-finite rejection
- Test bounds preservation

### Integration Tests
- Run 2D test functions (Bukin, Himmelblau, etc.)
- Verify organisms evolve m6 and m6_sigma
- Check that populations eventually converge
- Measure performance vs baseline (m6=0.0)

### Manual UI Tests
- Start server
- Run optimization
- Inspect organism details
- Verify m6, m6_sigma display correctly
- Check values are within bounds

---

## Rollback Plan

If issues arise:
1. All changes are in new parameters - easy to disable by setting m6=0.0 in parameter_enhancement.rs
2. Existing logic unchanged - backward compatible
3. Tests guard against regressions
4. Git branch allows clean revert

---

## Dependencies

**Rust Crates:**
- `rand_distr` - Already in dependencies (provides StandardNormal distribution)
- No new dependencies required

**Build Requirements:**
- Rust 1.70+ (existing requirement)
- No toolchain changes

---

## Questions for Clarification

This plan assumes:
1. ✅ m6 and m6_sigma should both be evolvable (confirmed)
2. ✅ Apply Gaussian noise AFTER m5 mutation (agreed)
3. ✅ Reject negative/non-finite results silently (standard pattern)
4. ✅ Initial values: m6=0.01 (1%), m6_sigma=0.1 (10%)
5. ✅ Bounds: m6=[0.0, 1.0], m6_sigma=[0.001, 1.0]

No outstanding questions - ready to implement!

---

## Success Criteria

Implementation is complete when:
- [ ] All existing tests pass with updated parameter counts
- [ ] New m6-specific tests pass with full coverage
- [ ] `cargo fmt && cargo clippy` produces no warnings
- [ ] PDD documentation is updated and accurate
- [ ] Web UI displays m6 and m6_sigma correctly
- [ ] Benchmarks show <5% performance impact
- [ ] Manual testing shows evolution of m6 and m6_sigma values
- [ ] Bukin N6 benchmark shows improved performance (lower scores)

---

**End of Implementation Plan**

This plan can be handed to a coding agent or developer for execution. All necessary context, file locations, code examples, and validation steps are included.
