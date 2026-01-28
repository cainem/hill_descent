import json
from collections import Counter
import sys

def analyze_profile(filename):
    print(f"Analyzing {filename}...")
    with open(filename, 'r') as f:
        data = json.load(f)
    
    frames = data['shared']['frames']
    frame_names = [f.get('name', 'unknown') for f in frames]
    
    total_samples = 0
    exclusive_counter = Counter()
    inclusive_counter = Counter()
    
    for profile in data['profiles']:
        samples = profile['samples']
        total_samples += len(samples)
        for stack in samples:
            if not stack:
                continue
            # Exclusive (leaf)
            exclusive_counter[stack[-1]] += 1
            # Inclusive (all in stack)
            unique_in_stack = set(stack)
            for frame_idx in unique_in_stack:
                inclusive_counter[frame_idx] += 1
                
    print(f"Total samples: {total_samples}")
    
    print(f"Top 20 frames by exclusive sample count for {filename}:")
    for frame_idx, count in exclusive_counter.most_common(20):
        print(f"{count:10d} ({100*count/total_samples:5.2f}%) : {frame_names[frame_idx]}")
        
    print(f"\nTop 20 frames by inclusive sample count for {filename}:")
    for frame_idx, count in inclusive_counter.most_common(20):
        print(f"{count:10d} ({100*count/total_samples:5.2f}%) : {frame_names[frame_idx]}")

    print("\nSpecific checks (Inclusive %):")
    targets = [
        "__lll_lock_wake_private",
        "ArcSwap",
        "Mutex",
        "RwLock",
        "Rayon",
        "IndexMap",
        "shift_remove",
        "malloc",
        "cfree",
        "Vec::push",
        "collect",
        "alloc",
        "dealloc"
    ]
    
    for t in targets:
        matches = [(idx, name) for idx, name in enumerate(frame_names) if t.lower() in name.lower()]
        total_inc = sum(inclusive_counter[idx] for idx, name in matches)
        total_exc = sum(exclusive_counter[idx] for idx, name in matches)
        if total_inc > 0:
            print(f"{t:25} : Inc {total_inc:6d} ({100*total_inc/total_samples:5.2f}%), Exc {total_exc:6d} ({100*total_exc/total_samples:5.2f}%)")
    print("\n")

if __name__ == "__main__":
    analyze_profile("profile.speedscope1.json")
    analyze_profile("profile.speedscope3.json")
