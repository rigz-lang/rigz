import 'utils/day.rg'

(file, p1, p2) = day_io 2

lines = file.lines.map { |line| line.split(' ').map { |l| l.to_i } }

fn mut List.safe -> Bool
    mut nums = self;
    mut last = nums.shift
    let next = nums.shift
    let diff = last.abs_diff(*next);
    let increasing = last < next;

    return false if diff > 3 || diff == 0

    last = next;
    for next in nums
        if increasing && next > last {
            if next - last > 3 {
                return false;
            }
        } else if !increasing && last > next {
            if last - next > 3 {
                return false;
            }
        } else {
            return false;
        }
        last = next;
    end
    true
end