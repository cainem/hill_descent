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

Zones will then be allocated a proportion of the total carrying capacity based on their size (the number of regions in the zone).

The zone size will be used directly to determine the proportion of the total carrying capacity allocated to each zone.

So if there are 3 zones with sizes 2, 3 and 5 then the carrying capacities will be allocated in the ratio 2:3:5.

Once the zone carrying capacities have been calculated then the carrying capacity of each region in the zone will be calculated based on the min score in that region compared to the min score in all regions in the zone (in that same that it is now except that currently the zone is the whole set of regions).
