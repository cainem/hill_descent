## Updating the calculation of carrying capacities.

Currently the carrying capacity of a region is calculated in relation only to the min score achieved in that region.

What I want to do now is introduction the concept of zones.
Zones are being introduced in an attempt to stop the algorithm "racing down the hill" and missing out on potentially better solutions in other areas of the search space.

## Zones

A zone is a collection of regions (there may only be one) in which all regions in the zones are adjacent to at least one other region in the zone.

The definition of adjacency to be used here is this one

where the Chebyshev distance between the points is 1.

$$D_C(P_a, P_b) = \max_{i} (|a_i - b_i|)$$

So with N regions the number of zones Z will be in the range 1 to N.
I will be looking to evaluate the minimum number of zones i.e. no two zones should be adjacent to each other.

Zones will then be allocated a proportion of the total carrying capacity using a hybrid approach that balances exploitation and exploration:

## Hybrid Zone Allocation

The total carrying capacity is split into two configurable funds controlled by the `FRACTIONAL_ZONE_ALLOCATION` constant:

1. **Global Fund**: Allocated based on zone scores, treating all regions as competing globally
   - Zone score = sum of (1/min_score) for all regions in the zone
   - Zones with better performing regions get more capacity from this fund

2. **Zone-Proportional Fund**: Allocated proportionally to zone sizes
   - Zone size = number of regions in the zone
   - Ensures fair representation across the search space regardless of performance

### Configuration

The `FRACTIONAL_ZONE_ALLOCATION` constant (scoped to the `Regions` struct) controls the split:
- **0.0**: All capacity allocated based on global score performance (pure exploitation)
- **1.0**: All capacity allocated proportionally to zone sizes (pure exploration)  
- **0.5**: Equal split between global and zone-proportional allocation (current default)

This allows fine-tuning the balance between exploitation and exploration in the algorithm.

### Example

If there are 3 zones with sizes 2, 3, and 5, and zone scores of 10.0, 20.0, and 30.0:

**With FRACTIONAL_ZONE_ALLOCATION = 0.5 (current default):**
- Total capacity: 100
- Global fund: 50 (allocated as 50×10/60=8.33≈8, 50×20/60=16.67≈17, 50×30/60=25)
- Zone-proportional fund: 50 (allocated as 50×2/10=10, 50×3/10=15, 50×5/10=25)
- Final allocations: [8+10=18, 17+15=32, 25+25=50]

**With FRACTIONAL_ZONE_ALLOCATION = 0.0 (pure exploitation):**
- Global fund: 100 (allocated as 100×10/60≈17, 100×20/60≈33, 100×30/60=50)
- Zone-proportional fund: 0
- Final allocations: [17, 33, 50]

**With FRACTIONAL_ZONE_ALLOCATION = 1.0 (pure exploration):**
- Global fund: 0
- Zone-proportional fund: 100 (allocated as 100×2/10=20, 100×3/10=30, 100×5/10=50)
- Final allocations: [20, 30, 50]

## Within-Zone Distribution

Once zone carrying capacities are calculated, the capacity within each zone is distributed based on the relative performance (min_scores) of regions within that zone only, using the same inverse fitness formula as before but scoped to the zone.
