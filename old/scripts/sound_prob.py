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

def s(p, pad=True):
    if p > 100.0:
        return "100.0"
    elif pad:
        return f"{p:5.1f}"
    else:
        return f"{p:.1f}"

if options.summarize:
    for dist in range(1, options.radius):
        p = options.volume/math.pow(dist, 1.2)
        print(f"{s(p, pad=False)} ", end = '')
    print("")
else:
    for dist in range(1, options.radius+1):
        p1 = options.volume*math.pow(0.75, dist)
        p2 = options.volume/(dist*dist)
        p3 = options.volume/math.pow(dist, 1.2)
        print(f"{dist:>2}: {s(p1)} {s(p2)} {s(p3)}")
