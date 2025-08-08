import 'utils/day.rg'

(file, p1, p2) = day_io 1

mut lhs = []
mut rhs = []

for line in file.lines
    (l, r) = line.split '   '
    lhs.push l.to_i
    rhs.push r.to_i
end

lhs.sort
rhs.sort

fn part1
    mut res = 0
    for l, r in lhs.zip rhs
        res += r > l ? r - l : l - r
    end
    res
end

vals = rhs.group_by(|v| v)

fn part2 = lhs.map {|l| l * ((vals[l] || []) as List).len }.sum

o1 = part1
o2 = part2

printf "part 1: {}", o1
printf "part 2: {}", o2

try assert_eq o1, p1.to_i
try assert_eq o2, p2.to_i