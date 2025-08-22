# DEPRECATED: RegionKeyDispenser

This document has been renamed. Please refer to `region_handle_dispenser.md` for the updated design and terminology:

- RegionKeyDispenser -> RegionHandleDispenser
- RegionKey -> RegionHandle
- adjacent_zone_key -> adjacent_zone_id
- full key -> full_region_key

The content below is kept temporarily for historical context.

The `RegionKeyDispenser` is a component that manages the keys for regions in a multi-dimensional space.
It provides a way to efficiently retrieve and manage region keys, which are used to identify regions in the simulation.

## Overview

The number of dimensions can be very large therefore the full key for an organism can be very large as it is a Vec<usize> where the number of elements is the number of dimensions.
The Vec<usize> is a poor key as it so large in practice the number of keys in use will be limited to the number of regions therefore it is more efficient to have a smaller key `RegionKey` that maps to the full key in the `RegionKeyDispenser`

In addition to the key the `RegionKey` will also contain the adjacent_zone_key
The key denotes which adjacent zone the key belongs to.
An adjacent zone is a collection of regions in space where all of the regions are connected to at least one other region in the zone by an adjacent face.
An adjacent face is defined such that given the full keys of two regions 
a=[a1, a2, a3, a4 ...]  & b=[b1, b2, b3, b4, ...] 
the keys are such that all coordinates are equal (so a1=b1, a2=b2 etc) except for 1 coordinate and that 1 coordinate can only differ numerically by 1 from its counterpart.

So in the example above these regions would be adjacent a=[4, 5, 6, 7, 8] & b=[4, 5, 6, 7, 9], so 4=4, 5=5, 6=6, 7=7 & 8+1=9

## Start of processing

When the regions get recalculated all of the keys in the key dispenser become invalid.
The `RegionKeyDispenser` will be reset when this happens. Its internal btree will be cleared and the next_key and next_adjacent_zone_key will be reset to 0.

The first organism will then go to the dispenser to get a region key it will present its full key which is a Vec<usize>.
The dispenser will then check if the key is already in the btree, if it is not it will create a new `RegionKey` with the next key and the next adjacent zone key.
In this case the btree will be empty so the next key will be 0 and the next adjacent zone key will also be 0. These will be used to construct the `RegionKey`.
The `RegionKey` will then be inserted into the btree with the full key as the key and the `RegionKey` as the value, this value will be returned to the caller. The next key and next adjacent zone key will then be incremented by 1.

The next organism will then present its full key to the dispenser.

If the full key is already in the btree the `RegionKey` will be returned to the caller.

If the full key is not in the btree, a new `RegionKey` will be created with the next key value (now 1).
The adjacent zone key however will be more complicated.

## Adjacent Zone Key Calculation

When a full key is presented to the dispenser, then if that full key has not been seen before it needs to work out which adjacent zone it belongs to before creating a new `RegionKey`.

It will need to compare itself to each full key that is already in the btree.

If, using the definition of adjacent face above, it is adjacent to one of the full keys in the btree then it will be assigned the same adjacent zone key as that full key.

If it is not adjacent to any of the full keys in the btree then it will be assigned a new adjacent zone key which will be the next adjacent zone key. The next adjacent zone key will then be incremented by 1.

If it is adjacent to more than one full key in the btree then if those regions are in the same adjacent zone then it will be assigned the same adjacent zone key as those regions.

However if any of the adjacent regions are in different adjacent zones then the adjacent zones will need to be merged.

## Adjacent Zone Merging

Say we have a full key `a` and it found to be adjacent to two full keys `b` and `c` in the btree.
If `b` in in adjacent zone 0 and `c` is in adjacent zone 1 then the dispenser will need to merge the two adjacent zones.
To do this it will need to iterate through the btree and find all of the full keys that are in adjacent zone 1 and change their adjacent zone key to 0.
`a` will also be assigned to adjacent zone 0.

Picking 0 as the adjacent zone key is arbitrary, it could have easily been 1.
This means the adjacent zone key 1 is lost and will never be used again. No attempt is made to reuse adjacent zone keys.

Also there could easily be more that two adjacent zones that need to be merged, in which case all of the adjacent zones will be merged into one adjacent zone.


## The end of processing

At the end of the process a region key will have been created for each region that is in use. 
There will be 1 or more adjacent zones, each with a unique adjacent zone key.


## Motivation for zones of adjacent regions

Although not implemented yet the idea here is to share the "available" carrying capacity amongst zones in an attempt to not rule out possible alternative solution too quickly.

At the moment the carrying capacity is calculated solely based in the min score of a region.

Going forward the idea will be to first divide the available carrying capacity between zones based on the number of regions in each zone.
Then within each zone the carrying capacity will be divided between the regions based on their min score.

I am thinking of using the square of the zone size to determine the portion of the carrying capacity that each zone gets.

so if there are 3 zones, A, B and C with sizes 1, 2 and 3 respectively 
then 1^2 + 2^2 + 3^2 = 14.

so zone A will get 1/14 of the carrying capacity, zone B will get 4/14 and zone C will get 9/14.

Then within each zone the regions will be assigned a portion of the carrying capacity based on their min score.

