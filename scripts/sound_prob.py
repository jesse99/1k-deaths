#!/opt/local/bin/python3.6
import argparse, math

parser = argparse.ArgumentParser(description='Print wake-up probabilities based on base volume.')
parser.add_argument('--radius', metavar='N', type=int, default=16,
    help='Max distance to use')
parser.add_argument('--summarize', action='store_true',
    help='Print a line of probabilities instead of a table')
parser.add_argument('--volume', metavar='N', type=int, default=100,
    help='Volume for NPCs right on top of the noise')

options = parser.parse_args()

for i in range(1, options.radius):
    p = options.volume*math.pow(0.75, i)
    if options.summarize:
        if p > 100.0:
            print(f"{100.0:.1f} ", end = '')
        else:
            print(f"{p:.1f} ", end = '')
    else:
        if p > 100.0:
            print(f"{i}: {100.0:.1f}")
        else:
            print(f"{i}: {p:.1f}")
if options.summarize:
    print("")
