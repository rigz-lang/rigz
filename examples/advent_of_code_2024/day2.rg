import 'utils/day.rg'

(file, p1, p2) = day_io 2

lines = (file.lines).map { |line| line.split(' ').map { |l| l.to_i } }

fn List.safe -> Bool
    mut nums = self.clone;
    mut last = nums.shift
    let n = nums.shift
    let diff = (last - n).abs
    let increasing = last < n;

    return :first if (diff > 3 || diff == 0)

    last = n;
    for n in nums
        if increasing && n > last
            return :second if n - last > 3
        else
            if !increasing && last > n
                return :third if (last - n > 3)
            else
                return :forth
            end
        end
        last = next
    end
    true
end

fn part1 = [for v in lines: 1 if v.safe.is true ].len

fn part2
    mut result = 0;

    for n: List in lines
        if n.safe
            result += 1
        else
            for index in n.enumerate
                let safe = n
                    .enumerate
                    .map { |v| v.1 if v.0 != index.0 }
                    .safe
                if safe
                    result += 1
                    break
                end
            end
        end
    end
    result
end

o1 = part1
o2 = part2

printf "part 1: {}", o1
printf "part 2: {}", o2

try assert_eq o1, p1.to_i
try assert_eq o2, p2.to_i