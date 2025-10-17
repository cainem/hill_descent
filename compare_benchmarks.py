import re

# Parse timing data from benchmark files
def parse_times(file_path):
    with open(file_path, 'r') as f:
        content = f.read()
    # Extract the results table
    match = re.search(r'\| Runs.*?\n\|(.*?)\n\n', content, re.DOTALL)
    if not match:
        return []
    lines = match.group(0).split('\n')[2:-2]  # Skip header and separator
    times = []
    for line in lines:
        parts = line.split('|')
        if len(parts) >= 9:
            pop = int(parts[2].strip())
            regions = int(parts[3].strip())
            time = float(parts[9].strip())
            times.append((pop, regions, time))
    return times

# Files to compare
old_files = [
    r'run_stats\2025-10\66e7cdab-16\20251016_065917_styblinski_tang.md',
    r'run_stats\2025-10\66e7cdab-16\20251016_070342_ackley.md',
    r'run_stats\2025-10\66e7cdab-16\20251016_070817_himmelblau.md',
]
new_files = [
    r'run_stats\2025-10\2a7273a6-16\20251016_074805_styblinski_tang.md',
    r'run_stats\2025-10\2a7273a6-16\20251016_075235_ackley.md',
    r'run_stats\2025-10\2a7273a6-16\20251016_075736_himmelblau.md',
]

names = ['Styblinski-Tang', 'Ackley', 'Himmelblau']

print('Performance Comparison: 66e7cdab (OLD) vs 2a7273a6 (NEW)')
print('='*80)

grand_total_old = 0
grand_total_new = 0

for name, old_file, new_file in zip(names, old_files, new_files):
    print(f'\n{name}:')
    print('-'*80)
    old_times = parse_times(old_file)
    new_times = parse_times(new_file)
    
    print(f"{'Pop':>6} {'Regions':>8} {'Old Time':>10} {'New Time':>10} {'Change':>10} {'% Change':>10}")
    print('-'*80)
    
    total_old = 0
    total_new = 0
    
    for (pop, reg, old_t), (_, _, new_t) in zip(old_times, new_times):
        change = new_t - old_t
        pct_change = (change / old_t * 100) if old_t > 0 else 0
        print(f'{pop:6d} {reg:8d} {old_t:10.3f}s {new_t:10.3f}s {change:+10.3f}s {pct_change:+9.1f}%')
        total_old += old_t
        total_new += new_t
    
    total_change = total_new - total_old
    total_pct = (total_change / total_old * 100) if total_old > 0 else 0
    print('-'*80)
    print(f"{'TOTAL':>6} {'':>8} {total_old:10.3f}s {total_new:10.3f}s {total_change:+10.3f}s {total_pct:+9.1f}%")
    
    grand_total_old += total_old
    grand_total_new += total_new

print('\n' + '='*80)
print('OVERALL SUMMARY')
print('='*80)
grand_change = grand_total_new - grand_total_old
grand_pct = (grand_change / grand_total_old * 100) if grand_total_old > 0 else 0
print(f"Total Old Time: {grand_total_old:10.3f}s")
print(f"Total New Time: {grand_total_new:10.3f}s")
print(f"Total Change:   {grand_change:+10.3f}s ({grand_pct:+.1f}%)")
