import File

fn day_io(day: Int) -> (String, String, String)
    input = File.read (format 'inputs/day{}.txt', day)
    p1 = File.read (format 'outputs/day{}_part1.txt', day)
    p2 = File.read (format 'outputs/day{}_part2.txt', day)
    (input, p1, p2)
end